use std::fs::{create_dir_all, File};
use std::path::{Path, PathBuf};

use uuid::Uuid;

pub fn build(sbom_path: impl AsRef<Path>, _cache: bool) -> anyhow::Result<()> {
    let sbom: anytree_sbom::CycloneDXBom =
        serde_json::from_reader(File::open(sbom_path.as_ref())?)?;

    let container_uuid = Uuid::new_v4().to_string();
    let container_name = format!("anytree-builder-{}", container_uuid);
    println!(
        r#"
||
|| Building container "{}"
||
"#,
        container_name
    );

    let builder_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from(".cache"))
        .join("anytree")
        .join("builder");
    tracing::trace!(?builder_dir, "Using builder directory");

    create_dir_all(&builder_dir)?;
    let builder_dir = builder_dir.canonicalize()?;
    tracing::trace!(?builder_dir, "Canonicalized builder directory");

    let container_dir = builder_dir.join(&container_name);
    create_dir_all(&container_dir)?;

    for component in &sbom.components {
        if let Some(properties) = &component.properties {
            if let Some(prop) = properties.iter().find(|prop| prop.name == "target") {
                if prop.value == anytree_plugin_cargo::PROJECT_TYPE {
                    tracing::trace!("Found cargo target");
                    anytree_plugin_cargo::build(component, &sbom, &container_name, container_dir)?;
                    return Ok(());
                }
            }
        }
    }

    anyhow::bail!("Failed to find valid target to build");
}
