use std::fs::File;
use std::path::Path;
use std::process::Command;

use uuid::Uuid;

pub fn build(sbom_path: impl AsRef<Path>) -> anyhow::Result<()> {
    let sbom: anytree_sbom::CycloneDXBom =
        serde_json::from_reader(File::open(sbom_path.as_ref())?)?;

    let container_uuid = Uuid::new_v4().to_string();
    tracing::info!(?container_uuid, "Building container");

    let container_name = format!("anytree-builder-{}", container_uuid);

    let mut docker_cmd = Command::new("docker");
    docker_cmd.arg("run");
    docker_cmd.arg("--network").arg("none");
    docker_cmd.arg("--name").arg(&container_name);
    for component in sbom.components {
        tracing::info!(?component, "Building component");
    }

    tracing::info!("Metadata: {}", sbom.metadata.unwrap().timestamp.unwrap().to_rfc2822());
    tracing::info!(?docker_cmd, "Running docker command");

    let output = docker_cmd.output()?;

    tracing::info!(?output, "Finished running docker command");
    Ok(())
}
