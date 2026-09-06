use std::ffi::c_void;
use std::mem::ManuallyDrop;
use std::path::{Path, PathBuf};
use std::ptr::{null, null_mut};

use windows::Win32::Foundation::{FreeLibrary, HMODULE, VARIANT_BOOL};
use windows::Win32::System::Com::{
    DISPATCH_METHOD, DISPATCH_PROPERTYGET, DISPPARAMS, IClassFactory, IDispatch,
};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
use windows::Win32::System::Variant::{
    VARIANT, VARIANT_0, VARIANT_0_0, VARIANT_0_0_0, VT_BOOL, VT_BYREF, VT_I4,
};
use windows::core::{GUID, HRESULT, Interface, PCSTR, PCWSTR};

const CLSID_WINMERGE_SCRIPT: GUID = GUID::from_u128(0xa4f619ac_6b9e_48a9_a09d_3b12200693ea);

type DllGetClassObject = unsafe extern "system" fn(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT;

struct LoadedLibrary(HMODULE);

impl Drop for LoadedLibrary {
    fn drop(&mut self) {
        unsafe {
            let _ = FreeLibrary(self.0);
        }
    }
}

struct TemporaryFile(PathBuf);

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn wide(value: &Path) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    value.as_os_str().encode_wide().chain(Some(0)).collect()
}

unsafe fn load_plugin() -> (LoadedLibrary, IClassFactory, IDispatch) {
    let dll_path = std::env::current_exe()
        .expect("the test executable path should be available")
        .parent()
        .expect("the test executable should have a parent directory")
        .join("compare_office_with_anydoc.dll");
    let dll_path_wide = wide(&dll_path);
    let module = unsafe { LoadLibraryW(PCWSTR(dll_path_wide.as_ptr())) }
        .unwrap_or_else(|error| panic!("failed to load {}: {error}", dll_path.display()));
    let library = LoadedLibrary(module);
    let proc = unsafe { GetProcAddress(module, PCSTR(c"DllGetClassObject".as_ptr().cast())) }
        .expect("DllGetClassObject should be exported");
    let get_class_object: DllGetClassObject = unsafe { std::mem::transmute(proc) };

    let mut factory_ptr = null_mut();
    unsafe {
        get_class_object(
            &CLSID_WINMERGE_SCRIPT,
            &IClassFactory::IID,
            &mut factory_ptr,
        )
        .ok()
        .expect("DllGetClassObject should return the plugin class factory");
    }
    let factory = unsafe { IClassFactory::from_raw(factory_ptr) };
    let plugin = unsafe { factory.CreateInstance::<_, IDispatch>(None) }
        .expect("the class factory should create an IDispatch instance");

    (library, factory, plugin)
}

unsafe fn dispid(plugin: &IDispatch, name: &str) -> i32 {
    let name: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
    let name = PCWSTR(name.as_ptr());
    let mut dispid = 0;
    unsafe {
        plugin
            .GetIDsOfNames(&GUID::zeroed(), &name, 1, 0, &mut dispid)
            .unwrap_or_else(|error| panic!("failed to resolve Automation name: {error}"));
    }
    dispid
}

unsafe fn property(plugin: &IDispatch, name: &str) -> VARIANT {
    let mut result = VARIANT::default();
    unsafe {
        plugin
            .Invoke(
                dispid(plugin, name),
                &GUID::zeroed(),
                0,
                DISPATCH_PROPERTYGET,
                &DISPPARAMS::default(),
                Some(&mut result),
                None,
                None,
            )
            .unwrap_or_else(|error| panic!("failed to read {name}: {error}"));
    }
    result
}

fn variant_string(value: &VARIANT) -> String {
    let bstr = unsafe { &*value.Anonymous.Anonymous.Anonymous.bstrVal };
    bstr.to_string()
}

fn byref_bool(value: &mut VARIANT_BOOL) -> VARIANT {
    VARIANT {
        Anonymous: VARIANT_0 {
            Anonymous: ManuallyDrop::new(VARIANT_0_0 {
                vt: VT_BYREF | VT_BOOL,
                wReserved1: 0,
                wReserved2: 0,
                wReserved3: 0,
                Anonymous: VARIANT_0_0_0 { pboolVal: value },
            }),
        },
    }
}

fn byref_i32(value: &mut i32) -> VARIANT {
    VARIANT {
        Anonymous: VARIANT_0 {
            Anonymous: ManuallyDrop::new(VARIANT_0_0 {
                vt: VT_BYREF | VT_I4,
                wReserved1: 0,
                wReserved2: 0,
                wReserved3: 0,
                Anonymous: VARIANT_0_0_0 { plVal: value },
            }),
        },
    }
}

#[test]
fn dll_exposes_a_working_winmerge_plugin() {
    let (_library, factory, plugin) = unsafe { load_plugin() };

    assert_eq!(
        variant_string(&unsafe { property(&plugin, "PluginEvent") }),
        "FILE_PACK_UNPACK"
    );
    assert_eq!(
        variant_string(&unsafe { property(&plugin, "PluginDescription") }),
        "Office document comparison using anydoc"
    );
    assert!(bool::try_from(&unsafe { property(&plugin, "PluginIsAutomatic") }).unwrap());
    assert_eq!(
        variant_string(&unsafe { property(&plugin, "PluginUnpackedFileExtension") }),
        ".md"
    );

    let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/com-plugin/test-com-plugin.docx");
    let output = TemporaryFile(std::env::temp_dir().join(format!(
        "compare-office-with-anydoc-{}-{}.md",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    )));
    let mut changed = VARIANT_BOOL::default();
    let mut subcode = -1;
    let mut args = [
        byref_i32(&mut subcode),
        byref_bool(&mut changed),
        VARIANT::from(output.0.to_string_lossy().as_ref()),
        VARIANT::from(input.to_string_lossy().as_ref()),
    ];
    let params = DISPPARAMS {
        rgvarg: args.as_mut_ptr(),
        rgdispidNamedArgs: null_mut(),
        cArgs: args.len() as u32,
        cNamedArgs: 0,
    };
    let mut result = VARIANT::default();

    unsafe {
        plugin
            .Invoke(
                dispid(&plugin, "UnpackFile"),
                null(),
                0,
                DISPATCH_METHOD,
                &params,
                Some(&mut result),
                None,
                None,
            )
            .expect("UnpackFile should be invoked successfully");
    }

    assert!(bool::try_from(&result).expect("UnpackFile should return a Boolean"));
    assert!(bool::from(changed));
    assert_eq!(subcode, 0);
    assert!(
        std::fs::metadata(&output.0)
            .expect("UnpackFile should create the Markdown output")
            .len()
            > 0,
        "the generated Markdown should not be empty"
    );

    drop(plugin);
    drop(factory);
}
