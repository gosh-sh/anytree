mod cargo_components;

use std::path::Path;
use crate::cargo_components::parse_component;
use anytree_sbom::CycloneDXBom;

const COMPONENT_TYPE: &str = "Library";

pub fn load_dependencies(
    sbom: impl AsRef<CycloneDXBom>,
    cargo_dir: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let bom = sbom.as_ref();

    // prepare project dependencies
    for dependency in &bom.components {
        if dependency.component_type == COMPONENT_TYPE {
            parse_component(dependency, cargo_dir.as_ref())?;
        }
    }

    Ok(())
}
