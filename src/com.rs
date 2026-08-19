use std::ffi::c_void;
use std::path::Path;
use std::ptr::null_mut;

use windows::Win32::Foundation::{
    CLASS_E_CLASSNOTAVAILABLE, CLASS_E_NOAGGREGATION, DISP_E_BADINDEX, DISP_E_BADPARAMCOUNT,
    DISP_E_MEMBERNOTFOUND, DISP_E_TYPEMISMATCH, E_POINTER, HMODULE, VARIANT_BOOL,
};
use windows::Win32::System::Com::{
    DISPATCH_FLAGS, DISPPARAMS, EXCEPINFO, IClassFactory, IClassFactory_Impl, IDispatch,
    IDispatch_Impl, ITypeInfo,
};
use windows::Win32::System::LibraryLoader::{
    GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
    GetModuleFileNameW, GetModuleHandleExW,
};
use windows::Win32::System::Ole::{LoadTypeLibEx, REGKIND_NONE};
use windows::Win32::System::Variant::{VARIANT, VT_BOOL, VT_BSTR, VT_BYREF, VT_I4};
use windows::core::{
    BOOL, Error, GUID, HRESULT, IUnknown, Interface, PCWSTR, Ref, Result, implement,
};

pub const IID_IWINMERGE_SCRIPT: GUID = GUID::from_u128(0x2313ec64_fec5_47cf_8e01_ee3306f3bba1);

pub const CLSID_WINMERGE_SCRIPT: GUID = GUID::from_u128(0xa4f619ac_6b9e_48a9_a09d_3b12200693ea);

const DISPID_PLUGIN_EVENT: i32 = 1;
const DISPID_PLUGIN_DESCRIPTION: i32 = 2;
const DISPID_PLUGIN_FILE_FILTERS: i32 = 3;
const DISPID_PLUGIN_IS_AUTOMATIC: i32 = 4;
const DISPID_PLUGIN_UNPACKED_FILE_EXTENSION: i32 = 5;
const DISPID_UNPACK_FILE: i32 = 6;
const DISPID_PACK_FILE: i32 = 7;

fn load_plugin_type_info() -> Result<ITypeInfo> {
    // Load the type library embedded in this DLL so Automation metadata remains
    // available without registering the plugin in the system registry.
    let mut module = HMODULE::default();

    unsafe {
        GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            PCWSTR(DllGetClassObject as *const () as *const u16),
            &mut module,
        )?;
    }

    let mut path = vec![0u16; 32768];

    let len = unsafe { GetModuleFileNameW(Some(module), &mut path) };

    if len == 0 {
        return Err(Error::from_thread());
    }

    path.truncate(len as usize);
    path.push(0);

    let type_lib = unsafe { LoadTypeLibEx(PCWSTR(path.as_ptr()), REGKIND_NONE)? };

    unsafe { type_lib.GetTypeInfoOfGuid(&IID_IWINMERGE_SCRIPT) }
}

fn variant_to_string(value: &VARIANT) -> Result<String> {
    if value.vt() != VT_BSTR {
        return Err(Error::from_hresult(DISP_E_TYPEMISMATCH));
    }

    let bstr = unsafe { &*value.Anonymous.Anonymous.Anonymous.bstrVal };

    String::from_utf16(bstr).map_err(|_| Error::from_hresult(DISP_E_TYPEMISMATCH))
}

fn byref_bool(value: &VARIANT) -> Result<*mut VARIANT_BOOL> {
    if value.vt() != (VT_BYREF | VT_BOOL) {
        return Err(Error::from_hresult(DISP_E_TYPEMISMATCH));
    }

    let ptr = unsafe { value.Anonymous.Anonymous.Anonymous.pboolVal };

    if ptr.is_null() {
        return Err(Error::from_hresult(E_POINTER));
    }

    Ok(ptr)
}

fn byref_i32(value: &VARIANT) -> Result<*mut i32> {
    if value.vt() != (VT_BYREF | VT_I4) {
        return Err(Error::from_hresult(DISP_E_TYPEMISMATCH));
    }

    let ptr = unsafe { value.Anonymous.Anonymous.Anonymous.plVal };

    if ptr.is_null() {
        return Err(Error::from_hresult(E_POINTER));
    }

    Ok(ptr)
}

fn dispatch_args(params: &DISPPARAMS, expected: usize) -> Result<&[VARIANT]> {
    if params.cArgs as usize != expected {
        return Err(Error::from_hresult(DISP_E_BADPARAMCOUNT));
    }

    if params.rgvarg.is_null() {
        return Err(Error::from_hresult(E_POINTER));
    }

    // The count and pointer must be validated before treating caller-owned COM
    // memory as a Rust slice.
    Ok(unsafe { std::slice::from_raw_parts(params.rgvarg, params.cArgs as usize) })
}

fn set_result(result: *mut VARIANT, value: VARIANT) -> Result<()> {
    if result.is_null() {
        return Err(Error::from_hresult(E_POINTER));
    }

    unsafe {
        *result = value;
    }

    Ok(())
}

fn invoke_unpack_file(params: *const DISPPARAMS, result: *mut VARIANT) -> Result<()> {
    let params = unsafe {
        params
            .as_ref()
            .ok_or_else(|| Error::from_hresult(E_POINTER))?
    };

    /*
     * IDispatch arguments are provided in reverse order.
     *
     * IDL:
     *   UnpackFile(
     *       fileSrc,
     *       fileDst,
     *       pbChanged,
     *       pSubcode
     *   )
     *
     * DISPPARAMS:
     *   args[0] = pSubcode
     *   args[1] = pbChanged
     *   args[2] = fileDst
     *   args[3] = fileSrc
     */
    let args = dispatch_args(params, 4)?;

    let subcode = byref_i32(&args[0])?;
    let changed = byref_bool(&args[1])?;

    let file_dst = variant_to_string(&args[2])?;
    let file_src = variant_to_string(&args[3])?;

    // Once the Automation call is valid, report conversion failures through the
    // WinMerge return value rather than as an IDispatch failure.
    let success = crate::winmerge::unpack_file(Path::new(&file_src), Path::new(&file_dst)).is_ok();

    unsafe {
        *subcode = 0;
        *changed = VARIANT_BOOL::from(success);
    }

    set_result(result, VARIANT::from(success))
}

fn invoke_pack_file(params: *const DISPPARAMS, result: *mut VARIANT) -> Result<()> {
    let params = unsafe {
        params
            .as_ref()
            .ok_or_else(|| Error::from_hresult(E_POINTER))?
    };

    /*
     * IDL:
     *   PackFile(
     *       fileSrc,
     *       fileDst,
     *       pbChanged,
     *       pSubcode
     *   )
     *
     * DISPPARAMS:
     *   args[0] = pSubcode
     *   args[1] = pbChanged
     *   args[2] = fileDst
     *   args[3] = fileSrc
     */
    let args = dispatch_args(params, 4)?;

    // Validate unused inputs as well so malformed Automation calls receive the
    // standard COM type-mismatch error.
    if args[0].vt() != VT_I4 {
        return Err(Error::from_hresult(DISP_E_TYPEMISMATCH));
    }

    let changed = byref_bool(&args[1])?;

    if args[2].vt() != VT_BSTR || args[3].vt() != VT_BSTR {
        return Err(Error::from_hresult(DISP_E_TYPEMISMATCH));
    }

    /*
     * This plugin only converts from Office documents to Markdown.
     * It does not repack Markdown into Office documents.
     */
    unsafe {
        *changed = VARIANT_BOOL::from(false);
    }

    set_result(result, VARIANT::from(false))
}

#[implement(IDispatch)]
pub struct WinMergeScript;

impl IDispatch_Impl for WinMergeScript_Impl {
    fn GetTypeInfoCount(&self) -> Result<u32> {
        Ok(1)
    }

    fn GetTypeInfo(&self, itinfo: u32, _lcid: u32) -> Result<ITypeInfo> {
        if itinfo != 0 {
            return Err(Error::from_hresult(DISP_E_BADINDEX));
        }

        load_plugin_type_info()
    }

    fn GetIDsOfNames(
        &self,
        _riid: *const GUID,
        rgsznames: *const PCWSTR,
        cnames: u32,
        _lcid: u32,
        rgdispid: *mut i32,
    ) -> Result<()> {
        // Keep plugin.idl as the source of truth for name-to-DISPID mapping.
        let type_info = load_plugin_type_info()?;

        unsafe { type_info.GetIDsOfNames(rgsznames, cnames, rgdispid) }
    }

    fn Invoke(
        &self,
        dispidmember: i32,
        _riid: *const GUID,
        _lcid: u32,
        _wflags: DISPATCH_FLAGS,
        pdispparams: *const DISPPARAMS,
        pvarresult: *mut VARIANT,
        _pexcepinfo: *mut EXCEPINFO,
        _puargerr: *mut u32,
    ) -> Result<()> {
        // These DISPIDs must stay aligned with plugin.idl and the embedded type library.
        match dispidmember {
            DISPID_PLUGIN_EVENT => set_result(pvarresult, VARIANT::from("FILE_PACK_UNPACK")),

            DISPID_PLUGIN_DESCRIPTION => set_result(
                pvarresult,
                VARIANT::from("Office document comparison using anydoc"),
            ),

            DISPID_PLUGIN_FILE_FILTERS => set_result(
                pvarresult,
                VARIANT::from(
                    r"\.(doc|docx|docm|xls|xlsx|xlsm|xlsb|ppt|pps|pot|pptx|pptm|ppsx|ppsm)$",
                ),
            ),

            DISPID_PLUGIN_IS_AUTOMATIC => set_result(pvarresult, VARIANT::from(true)),

            DISPID_PLUGIN_UNPACKED_FILE_EXTENSION => set_result(pvarresult, VARIANT::from(".md")),

            DISPID_UNPACK_FILE => invoke_unpack_file(pdispparams, pvarresult),

            DISPID_PACK_FILE => invoke_pack_file(pdispparams, pvarresult),

            _ => Err(Error::from_hresult(DISP_E_MEMBERNOTFOUND)),
        }
    }
}

#[implement(IClassFactory)]
struct WinMergeClassFactory;

impl IClassFactory_Impl for WinMergeClassFactory_Impl {
    fn CreateInstance(
        &self,
        punkouter: Ref<'_, IUnknown>,
        riid: *const GUID,
        ppvobject: *mut *mut c_void,
    ) -> Result<()> {
        // This lightweight factory does not implement COM object aggregation.
        if punkouter.is_some() {
            return Err(Error::from_hresult(CLASS_E_NOAGGREGATION));
        }

        if riid.is_null() || ppvobject.is_null() {
            return Err(Error::from_hresult(E_POINTER));
        }

        // COM requires output interface pointers to be null on failure.
        unsafe {
            *ppvobject = null_mut();
        }

        let object: IDispatch = WinMergeScript.into();

        unsafe { object.query(riid, ppvobject) }.ok()
    }

    fn LockServer(&self, _flock: BOOL) -> Result<()> {
        // WinMerge owns the DLL lifetime; the plugin keeps no separate lock count.
        Ok(())
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if rclsid.is_null() || riid.is_null() || ppv.is_null() {
        return E_POINTER;
    }

    // Clear the caller's output before any class or interface lookup can fail.
    unsafe {
        *ppv = null_mut();

        if *rclsid != CLSID_WINMERGE_SCRIPT {
            return CLASS_E_CLASSNOTAVAILABLE;
        }
    }

    let factory: IClassFactory = WinMergeClassFactory.into();

    unsafe { factory.query(riid, ppv) }
}
