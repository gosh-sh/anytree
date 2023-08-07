mod cargo_components;

use std::path::Path;

use anytree_sbom::{ComponentType, CycloneDXBom};
use anytree_utils::tracing::{increase_progress, start_progress};

use crate::cargo_components::parse_component;

pub fn load_dependencies(sbom: &CycloneDXBom, cargo_dir: impl AsRef<Path>) -> anyhow::Result<()> {
    // prepare project dependencies
    let length = sbom.components.len();
    let span = start_progress(length as u64);
    let span_enter = span.enter();
    for component in &sbom.components {
        if component.component_type == ComponentType::Library {
            parse_component(component, cargo_dir.as_ref())?;
            increase_progress();
        }
    }
    drop(span_enter);
    drop(span);
    Ok(())
}
