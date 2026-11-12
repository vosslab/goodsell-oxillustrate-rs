use clap::Parser;

#[derive(Parser)]
#[command(
    name = "illustrate",
    version,
    about = "Native molecular illustration",
    arg_required_else_help = true
)]
struct Arguments {}

pub fn run() {
    let _arguments = Arguments::parse();
}
