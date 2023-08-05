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
    let cli = Cli::try_parse()?;

    match cli.command {
        Commands::Build { sbom , dir} => {
            if !sbom.exists() {
                anyhow::bail!("{sbom:?} does not exist");
            }

            // TODO: cache
            anytree_cli::commands::build::build(sbom, dir)?;
        }
    }

    Ok(())
}
