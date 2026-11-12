//! Command-line entry point for the `illustrate` renderer.
#![forbid(unsafe_code)]
mod command_line;

fn main() -> std::process::ExitCode {
    match command_line::run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("illustrate: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
