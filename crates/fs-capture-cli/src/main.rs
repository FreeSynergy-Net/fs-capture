//! `fs-capture` command line. Built with help from Claude (AI).

mod args;
mod run;

use std::process::ExitCode;

use fs_capture::{CaptureError, CaptureSource};

#[cfg(feature = "xcap")]
fn source() -> Result<Box<dyn CaptureSource>, CaptureError> {
    Ok(Box::new(fs_capture::XcapSource))
}

#[cfg(not(feature = "xcap"))]
fn source() -> Result<Box<dyn CaptureSource>, CaptureError> {
    Err(CaptureError::new(
        fs_capture::ErrorCode::Backend,
        "this build has no capture backend (built without feature `xcap`)",
    ))
}

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let inv = match args::parse(&argv) {
        Ok(i) => i,
        Err(msg) => {
            eprintln!("error[E_USAGE]: {msg}\n\n{}", args::HELP);
            return ExitCode::from(2);
        }
    };
    let mut out = std::io::stdout().lock();
    let result = source().and_then(|s| run::run(&inv, s.as_ref(), &mut out));
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error[{}]: {}", e.code().as_str(), e.message());
            ExitCode::from(1)
        }
    }
}
