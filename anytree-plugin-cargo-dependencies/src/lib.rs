mod cargo_components;
pub mod crypto;

use std::path::Path;

use anytree_sbom::{ComponentType, CycloneDXBom};

use crate::cargo_components::parse_component;

pub fn load_dependencies(sbom: &CycloneDXBom, cargo_dir: impl AsRef<Path>) -> anyhow::Result<()> {
    // prepare project dependencies
    for component in &sbom.components {
        if component.component_type == ComponentType::Library {
            parse_component(component, cargo_dir.as_ref())?;
        }
    }

    Ok(())
}
