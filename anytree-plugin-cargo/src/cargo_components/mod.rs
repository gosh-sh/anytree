use std::path::Path;

use anytree_sbom::Component;

use crate::cargo_components::git::CargoGitComponent;
use crate::cargo_components::registry::CargoRegistryComponent;

mod git;
mod helper;
mod registry;

pub fn parse_component(component: &Component, cargo_root: impl AsRef<Path>) -> anyhow::Result<()> {
    // parse dependency properties
    tracing::trace!("Component type: {}", &component.mime_type);
    match component.mime_type.as_str() {
        registry::LIBRARY_TYPE => {
            CargoRegistryComponent::save(cargo_root.as_ref(), component)?;
        }
        git::LIBRARY_TYPE => {
            CargoGitComponent::save(cargo_root.as_ref(), component)?;
        }
        other => {
            panic!("Unsupported type of component library: {}", other);
        }
    }
    Ok(())
}
