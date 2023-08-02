use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anytree_sbom::{Component, CycloneDXBom};
use anytree_utils::crypto::hash::check_hashes;
use anytree_utils::tracing::wrap_cmd_with_tracing;

pub const PROJECT_TYPE: &str = "bash_script";
const BASE_CONTAINER: &str = "ubuntu:22.04";

pub fn execute(
    bash_component: &Component,
    sbom: &CycloneDXBom,
    container_name: &str,
    run_dir: impl AsRef<Path>,
) -> anyhow::Result<()> {
    tracing::info!("Start bash script execution");
    let bash_cmd = bash_component
        .description
        .as_ref()
        .ok_or(anyhow::format_err!("bash component should contain script"))?;
    let workdir = &bash_component
        .properties
        .as_ref()
        .and_then(|properties| properties.iter().find(|property| property.name == "workdir"))
        .ok_or(anyhow::format_err!("Failed to get workdir for component: {}", bash_component.name))?
        .value;

    tracing::trace!("Prepare run container");
    let mut docker_cmd = Command::new("docker");
    docker_cmd.arg("run");
    docker_cmd.arg("-t");
    docker_cmd.arg("--network").arg("none");
    docker_cmd.arg("--name").arg(container_name);
    docker_cmd.arg("--workdir").arg(workdir);
    docker_cmd.arg(BASE_CONTAINER);
    docker_cmd.arg("/bin/bash").arg("-c");
    docker_cmd.arg(bash_cmd);

    docker_cmd.stdout(Stdio::piped());
    docker_cmd.stderr(Stdio::piped());

    // TODO: better wording
    tracing::info!("Running docker builder (hint: use TRACE level for more info)");
    tracing::trace!("Running docker command: {:?}", {
        let mut parts = vec![docker_cmd.get_program()];
        parts.extend(docker_cmd.get_args());
        parts.join(OsStr::new(" "))
    });

    let mut docker_child = docker_cmd.spawn()?;

    wrap_cmd_with_tracing(&mut docker_child);

    let res = docker_child.wait()?;
    if !res.success() {
        anyhow::bail!("docker command failed: {}", res.code().unwrap_or(-1));
    }

    let (artifact_name, artifact_hashes) = if let Some(component) =
        &sbom.metadata.as_ref().and_then(|metadata| metadata.component.as_ref())
    {
        (component.name.clone(), component.hashes.clone())
    } else {
        // TODO: load name from Cargo.toml
        ("name".to_string(), None)
    };

    let mut docker_cp = Command::new("docker");
    docker_cp.arg("cp");

    let mut container_path = OsString::from(container_name);
    container_path.push(":");
    container_path.push(workdir);
    container_path.push("/");
    container_path.push(&artifact_name);

    tracing::trace!("Container artifact path: {:?}", container_path);

    docker_cp.arg(container_path);
    docker_cp.arg(run_dir.as_ref());
    tracing::trace!(?docker_cp, "Running docker command");

    let res = docker_cp.status()?;
    if !res.success() {
        anyhow::bail!("docker command failed: {}", res.code().unwrap_or(-1));
    }

    if let Some(hashes) = artifact_hashes {
        tracing::info!("Check hash of the artifact");
        let mut target_path = PathBuf::from(run_dir.as_ref());
        target_path.push(&artifact_name);
        let data = std::fs::read(&target_path)?;
        check_hashes(&hashes, data)?;
        tracing::info!("Hash is valid");
    }

    Ok(())
}
