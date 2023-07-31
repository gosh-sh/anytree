use std::path::Path;

use anytree_sbom::Component;

use crate::cargo_components::git::CargoGitComponent;
use crate::cargo_components::registry::CargoRegistryComponent;

mod git;
mod hash;
mod helper;
mod registry;

pub fn parse_component(component: &Component, cargo_root: impl AsRef<Path>) -> anyhow::Result<()> {
    // parse dependency properties
    let component_type = component
        .mime_type
        .as_ref()
        .ok_or(anyhow::format_err!("Component {} does not contain mime-type.", component.name))?;
    tracing::trace!("Component type: {}", &component_type);
    match component_type.as_str() {
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
