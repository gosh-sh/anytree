use std::process::exit;

use anytree_cli::commands::{Cli, Commands};
use clap::Parser;

fn main() {
    anytree_utils::tracing::default_init();

    match run() {
        Ok(_) => {}
        Err(err) => {
            tracing::error!("{err}");

            exit(1);
        }
    };
}

fn run() -> anyhow::Result<()> {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            if e.to_string().starts_with("Usage") {
                eprintln!("{e}");
                return Ok(())
            }
            anyhow::bail!(e);
        }
    };

    match cli.command {
        Commands::Build { sbom, dir } => {
            if !sbom.exists() {
                anyhow::bail!("{sbom:?} does not exist");
            }

            // TODO: cache
            anytree_cli::commands::build::build(sbom, dir)?;
        }
    }

    Ok(())
}
