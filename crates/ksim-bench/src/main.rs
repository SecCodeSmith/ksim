use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "ksim-bench", version, about)]
struct Cli {}

fn main() {
    Cli::parse();
}
