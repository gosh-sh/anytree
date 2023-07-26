use crate::cargo_components::git::CargoGitComponent;
use crate::cargo_components::helper::get_component_properties;
use crate::cargo_components::registry::CargoRegistryComponent;
use std::path::Path;
use anytree_sbom::Component;

mod git;
mod helper;
mod registry;

pub fn parse_component(
    component: &Component,
    cargo_root: impl AsRef<Path>,
) -> anyhow::Result<()> {
    // parse dependency properties
    let properties = get_component_properties(&component)?;
    match properties["library_type"].as_str() {
        registry::LIBRARY_TYPE => {
            CargoRegistryComponent::save(cargo_root.as_ref(), &component)?;
        }
        git::LIBRARY_TYPE => {
            CargoGitComponent::save(cargo_root.as_ref(), &component)?;
        }
        other => {
            panic!("Unsupported type of component library: {}", other);
        }
    }
    Ok(())
}
