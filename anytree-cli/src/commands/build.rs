use std::ffi::OsString;
use std::fs::{create_dir_all, remove_dir_all, File};
use std::path::{Path, PathBuf};
use std::process::Command;

use anytree_plugin_cargo_dependencies::load_dependencies;
use uuid::Uuid;

pub fn build(sbom_path: impl AsRef<Path>, _cache: bool) -> anyhow::Result<()> {
    let sbom: anytree_sbom::CycloneDXBom =
        serde_json::from_reader(File::open(sbom_path.as_ref())?)?;

    let container_uuid = Uuid::new_v4().to_string();
    tracing::info!(?container_uuid, "Building container");

    let container_name = format!("anytree-builder-{}", container_uuid);

    let container_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from(".cache"))
        .join("anytree")
        .join("builder")
        .join(&container_name);

    for component in &sbom.components {
        if let Some(properties) = &component.properties {
            if let Some(prop) = properties.iter().find(|prop| prop.name == "target") {
                if prop.value == anytree_plugin_cargo::PROJECT_TYPE {
                    tracing::trace!("Found cargo target");
                    anytree_plugin_cargo::build_cargo_project(component, &sbom, container_dir)?;
                    return Ok(());
                }
            }
        }
    }

    anyhow::bail!("Failed to find valid target to build");
}

pub fn build_old(sbom_path: impl AsRef<Path>, cache: bool) -> anyhow::Result<()> {
    let sbom: anytree_sbom::CycloneDXBom =
        serde_json::from_reader(File::open(sbom_path.as_ref())?)?;

    let container_uuid = Uuid::new_v4().to_string();
    tracing::info!(?container_uuid, "Building container");

    let container_name = format!("anytree-builder-{}", container_uuid);

    let mut container_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from(".cache"))
        .join("gosh")
        .join("builder")
        .join(&container_name);

    if !cache {
        let _ = remove_dir_all(&container_dir); // ignore errors
        create_dir_all(&container_dir)?;
    }

    // NOTE: all paths must be absolute
    container_dir = container_dir.canonicalize()?;

    let cargo_dir = container_dir.join("cargo");
    if !cargo_dir.exists() {
        tracing::info!(?cargo_dir, "Creating directory");
        create_dir_all(&cargo_dir)?;
    }
    load_dependencies(&sbom, &cargo_dir)?;

    let src_dir = container_dir.join("src");
    if !src_dir.exists() {
        tracing::info!(?src_dir, "Creating directory");
        create_dir_all(&src_dir)?;
    }
    // TODO: read source git from SBOM
    let src_url = "https://github.com/gosh-sh/gosh-build-tools";
    load_git_repos(src_url, &src_dir)?;

    let mut docker_cmd = Command::new("docker");
    docker_cmd.arg("run");
    docker_cmd.arg("--network").arg("none");
    docker_cmd.arg("--name").arg(&container_name);

    // NOTE: keep in mind mount folders may contain OS specific characters
    docker_cmd.arg("--mount").arg({
        let mut s = OsString::from("type=bind,source=");
        s.push(cargo_dir.into_os_string());
        s.push(",target=/root/.cargo/");
        s
    });
    docker_cmd.arg("--mount").arg({
        let mut s = OsString::from("type=bind,source=");
        s.push(src_dir.into_os_string());
        s.push(",target=/root/src/");
        s
    });

    docker_cmd.arg("--workdir").arg("/root/src");
    docker_cmd.arg("rust:1.70");
    docker_cmd.arg("sh");
    docker_cmd.arg("-c");
    docker_cmd.arg("cargo build --offline --release");

    // tracing::info!("Metadata: {}",
    // sbom.metadata.unwrap().timestamp.unwrap().to_rfc2822());
    tracing::info!(?docker_cmd, "Running docker command");

    let output = docker_cmd.output()?;

    // let mut docker_cp_cmd = Command::new("docker");
    // docker_cp_cmd.arg("cp");

    // TODO: check HASH of the result build

    tracing::info!(?output, "Finished running docker command");
    if !output.status.success() {
        tracing::error!("docker command failed \n{}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}

fn load_git_repos(url: &str, src_dir: impl AsRef<Path>) -> anyhow::Result<()> {
    let mut cmd = Command::new("git");
    cmd.arg("clone");
    cmd.arg(url);
    cmd.arg(src_dir.as_ref());

    let output = cmd.output()?;
    if !output.status.success() {
        tracing::error!(?output, "git clone failed");
        anyhow::bail!("git clone failed");
    }
    Ok(())
}
