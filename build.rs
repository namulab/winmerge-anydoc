use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo::rerun-if-changed=plugin.idl");
    println!("cargo::rerun-if-changed=build.rs");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    #[cfg(windows)]
    build_windows();

    #[cfg(not(windows))]
    panic!("Building the WinMerge plugin requires a Windows host.");
}

#[cfg(windows)]
fn build_windows() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let idl = manifest_dir.join("plugin.idl");
    let tlb = out_dir.join("plugin.tlb");
    let rc = out_dir.join("plugin.rc");
    let res = out_dir.join("plugin.res");

    let vcvars = find_vcvars64();

    // plugin.idl -> plugin.tlb
    run_in_vs_environment(
        &vcvars,
        &format!(
            "midl /nologo /env x64 /out \"{}\" \"{}\"",
            out_dir.display(),
            idl.display()
        ),
    );

    // plugin.tlb を埋め込むリソーススクリプトを生成
    let tlb_for_rc = tlb.to_string_lossy().replace('\\', "\\\\");

    fs::write(&rc, format!("1 TYPELIB \"{tlb_for_rc}\"\r\n")).expect("failed to create plugin.rc");

    // plugin.rc -> plugin.res
    run_in_vs_environment(
        &vcvars,
        &format!("rc /nologo /fo \"{}\" \"{}\"", res.display(), rc.display()),
    );

    // plugin.res を cdylib にリンク
    println!("cargo::rustc-link-arg-cdylib={}", res.display());
}

#[cfg(windows)]
fn find_vcvars64() -> PathBuf {
    let program_files_x86 =
        env::var("ProgramFiles(x86)").expect("ProgramFiles(x86) is not defined");

    let vswhere = Path::new(&program_files_x86)
        .join("Microsoft Visual Studio")
        .join("Installer")
        .join("vswhere.exe");

    let output = Command::new(&vswhere)
        .args([
            "-latest",
            "-products",
            "*",
            "-requires",
            "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
            "-property",
            "installationPath",
        ])
        .output()
        .expect("failed to run vswhere.exe");

    if !output.status.success() {
        panic!(
            "vswhere.exe failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let installation_path = String::from_utf8_lossy(&output.stdout).trim().to_owned();

    if installation_path.is_empty() {
        panic!("Visual Studio with C++ build tools was not found");
    }

    let vcvars = Path::new(&installation_path)
        .join("VC")
        .join("Auxiliary")
        .join("Build")
        .join("vcvars64.bat");

    if !vcvars.exists() {
        panic!("vcvars64.bat was not found: {}", vcvars.display());
    }

    vcvars
}

#[cfg(windows)]
fn run_in_vs_environment(vcvars: &Path, command: &str) {
    use std::os::windows::process::CommandExt;

    let command_line = format!("call \"{}\" >nul && {}", vcvars.display(), command);

    /*
     * cmd.exe /c は通常のWindowsプログラムと
     * 引数の引用符処理が異なるため、
     * Command::arg() ではなく raw_arg() を使用する。
     *
     * cmd.exe が要求するように、
     * コマンド全体をさらに "..." で囲む。
     */
    let raw_command = format!("\"{}\"", command_line);

    let status = Command::new("cmd.exe")
        .arg("/d")
        .arg("/c")
        .raw_arg(raw_command)
        .status()
        .expect("failed to start Visual Studio command environment");

    if !status.success() {
        panic!("command failed: {}", command);
    }
}
