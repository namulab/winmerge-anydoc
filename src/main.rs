use std::env;
use std::path::Path;
use std::process::ExitCode;

use compare_office_with_anydoc::winmerge;

fn main() -> ExitCode {
    let bin_name = env!("CARGO_BIN_NAME");
    let mut args = env::args_os().skip(1);

    let Some(src) = args.next() else {
        eprintln!("Usage: {bin_name} <input-file> <output-file>");
        return ExitCode::FAILURE;
    };

    let Some(dst) = args.next() else {
        eprintln!("Usage: {bin_name} <input-file> <output-file>");
        return ExitCode::FAILURE;
    };

    if args.next().is_some() {
        eprintln!("Usage: {bin_name} <input-file> <output-file>");
        return ExitCode::FAILURE;
    }

    match winmerge::unpack_file(Path::new(&src), Path::new(&dst)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}
