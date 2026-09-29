use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "ksim-bench", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run a scenario with one policy over several seeds and write `<scenario>-<policy>.jsonl`
    /// and `.csv` into the output directory.
    Run {
        #[arg(long)]
        scenario: PathBuf,
        #[arg(long)]
        policy: String,
        #[arg(long, default_value_t = 10)]
        seeds: u32,
        #[arg(long, default_value_t = 0)]
        first_seed: u64,
        #[arg(long, default_value = "results")]
        out: PathBuf,
    },
    /// Print the available policy names, one per line.
    ListPolicies,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let _ = cli.command;
    todo!()
}
