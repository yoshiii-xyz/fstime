use clap::{Parser, Subcommand};
use fstime::{
    ProfileOptions, ProfileReport, compare_profiles, comparison_json, explain_profile,
    profile_json, profile_tree,
};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Debug, Parser)]
#[command(
    name = "fstime",
    version,
    about = "Profile explainable local filesystem traversal"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Profile one local tree and emit JSON.
    Profile {
        root: PathBuf,
        #[arg(long = "ignore")]
        ignore: Vec<String>,
        #[arg(long, default_value = "unspecified")]
        cache_label: String,
    },
    /// Compare two saved profile JSON reports.
    Compare { left: PathBuf, right: PathBuf },
    /// Explain one saved profile JSON report.
    Report { run: PathBuf },
}

fn main() -> ExitCode {
    match execute(Cli::parse()) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("fstime: {error}");
            ExitCode::from(2)
        }
    }
}

fn execute(cli: Cli) -> Result<u8, Box<dyn std::error::Error>> {
    match cli.command {
        Commands::Profile {
            root,
            ignore,
            cache_label,
        } => {
            let report = profile_tree(
                &root,
                &ProfileOptions {
                    ignore,
                    cache_label,
                },
            );
            println!("{}", profile_json(&report)?);
            Ok(if report.complete { 0 } else { 1 })
        }
        Commands::Compare { left, right } => {
            let left: ProfileReport = serde_json::from_str(&std::fs::read_to_string(left)?)?;
            let right: ProfileReport = serde_json::from_str(&std::fs::read_to_string(right)?)?;
            let comparison = compare_profiles(&left, &right);
            println!("{}", comparison_json(&comparison)?);
            Ok(0)
        }
        Commands::Report { run } => {
            let report: ProfileReport = serde_json::from_str(&std::fs::read_to_string(run)?)?;
            println!("{}", explain_profile(&report));
            Ok(0)
        }
    }
}
