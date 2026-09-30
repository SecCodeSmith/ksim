use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use anyhow::Context;
use clap::{CommandFactory, Parser, Subcommand};
use ksim_bench::{
    output::{write_csv, write_json_lines},
    policy::Policy,
    runner::run_seeds,
    scenario::Scenario,
};

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
    match Cli::parse().command {
        None => Cli::command().print_help().map_err(Into::into),
        Some(Command::ListPolicies) => {
            list_policies();
            Ok(())
        }
        Some(Command::Run {
            scenario,
            policy,
            seeds,
            first_seed,
            out,
        }) => run(&scenario, &policy, seeds, first_seed, &out),
    }
}

fn list_policies() {
    for policy in Policy::ALL {
        println!("{policy}");
    }
}

fn run(
    scenario_path: &Path,
    policy: &str,
    seeds: u32,
    first_seed: u64,
    out: &Path,
) -> anyhow::Result<()> {
    let policy: Policy = policy.parse()?;
    let scenario = Scenario::from_toml_file(scenario_path)?;
    let reports = run_seeds(&scenario, policy, first_seed, seeds)?;
    fs::create_dir_all(out).with_context(|| format!("creating {}", out.display()))?;
    let stem = format!("{}-{policy}", scenario.name);
    write_json_lines(&reports, File::create(out.join(format!("{stem}.jsonl")))?)?;
    write_csv(&reports, File::create(out.join(format!("{stem}.csv")))?)?;
    Ok(())
}
