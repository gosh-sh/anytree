use std::fs::File;
use std::process::Command;

use anytree_plugin_cargo::load_dependencies;
use anytree_sbom::CycloneDXBom;

fn main() -> anyhow::Result<()> {
    anytree_utils::tracing::default_init();
    let sbom_path = "hack/sbom.cdx.json";
    let sbom_file = File::open(sbom_path)?;
    let sbom: CycloneDXBom = serde_json::from_reader(sbom_file)?;

    let path = "/tmp/anytree-test-project/.gosh/cargo";
    Command::new("rm").arg("-rf").arg(path).status()?;
    load_dependencies(&sbom, path)?;
    Ok(())
}
