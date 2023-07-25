pub mod build;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]

pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]

pub enum Commands {
    Build {
        #[arg(name = "sbom_path")]
        sbom: PathBuf,
    },
}
