use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anytree_plugin_cargo_dependencies::crypto::hash::check_hashes;
use anytree_plugin_cargo_dependencies::load_dependencies;
use anytree_sbom::{Component, CycloneDXBom};

const PROJECT_DIR: &str = "src";
const DEPENDENCIES_DIR: &str = "cargo";
const TARGET_DIR: &str = "target";

const CONTAINER_PROJECT_DIR: &str = "/tmp/proj/";
const CONTAINER_REGISTRY_ROOT: &str = "/usr/local/cargo/";
const CONTAINER_TARGET_DIR: &str = "/tmp/target/";

const CONTAINER_BASE: &str = "rust:1.71";
const BUILD_CMD: &str = "cargo build --offline --release";

pub const PROJECT_TYPE: &str = "cargo/project";

pub fn build_cargo_project(
    project: &Component,
    sbom: &CycloneDXBom,
    run_dir: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let mut src_dir = PathBuf::from(run_dir.as_ref());
    src_dir.push(PROJECT_DIR);
    std::fs::create_dir_all(&src_dir)?;
    checkout_project(project, &src_dir)?;

    let mut deps_dir = PathBuf::from(run_dir.as_ref());
    deps_dir.push(DEPENDENCIES_DIR);
    std::fs::create_dir_all(&deps_dir)?;
    checkout_dependencies(sbom, &deps_dir)?;

    // prepare dir for artifact
    let mut target_dir = PathBuf::from(run_dir.as_ref());
    target_dir.push(TARGET_DIR);
    std::fs::create_dir_all(&target_dir)?;

    let container_name = format!("{}_{}", project.name, uuid::Uuid::new_v4());
    let mut docker_cmd = Command::new("docker");
    docker_cmd.arg("run");
    docker_cmd.arg("--network").arg("none");
    docker_cmd.arg("--name").arg(&container_name);

    // mount project dir
    docker_cmd.arg("--mount").arg({
        let mut s = OsString::from("type=bind,source=");
        s.push(src_dir.clone().into_os_string());
        s.push(",target=");
        s.push(CONTAINER_PROJECT_DIR);
        s
    });

    // mount git registry
    let mut git_registry = deps_dir.clone();
    git_registry.push("git");
    if git_registry.exists() {
        docker_cmd.arg("--mount").arg({
            let mut s = OsString::from("type=bind,source=");
            s.push(git_registry.into_os_string());
            s.push(",target=");
            s.push(CONTAINER_REGISTRY_ROOT);
            s.push("git");
            s
        });
    }

    // mount crates,io registry
    let mut crates_registry = deps_dir.clone();
    crates_registry.push("registry");
    if crates_registry.exists() {
        docker_cmd.arg("--mount").arg({
            let mut s = OsString::from("type=bind,source=");
            s.push(crates_registry.into_os_string());
            s.push(",target=");
            s.push(CONTAINER_REGISTRY_ROOT);
            s.push("registry");
            s
        });
    }

    // mount target dir
    let mut target_dir = PathBuf::from(run_dir.as_ref());
    target_dir.push(TARGET_DIR);
    std::fs::create_dir_all(&target_dir)?;
    docker_cmd.arg("--mount").arg({
        let mut s = OsString::from("type=bind,source=");
        s.push(target_dir.into_os_string());
        s.push(",target=");
        s.push(CONTAINER_TARGET_DIR);
        s
    });

    docker_cmd.arg("--workdir").arg(CONTAINER_PROJECT_DIR);
    docker_cmd.arg(CONTAINER_BASE);
    docker_cmd.arg("sh").arg("-c");

    let build_cmd = format!("{} --target-dir {}", BUILD_CMD, CONTAINER_TARGET_DIR);
    docker_cmd.arg(build_cmd);

    tracing::trace!(?docker_cmd, "Running docker command");

    let output = docker_cmd.output()?;
    tracing::trace!("Result: {:?}", output);

    // let mut docker_cp = Command::new("docker");
    // docker_cp.arg("cp");
    //
    // let container_path = format!("{}:{}", container_name, "/tmp/proj/target");
    // docker_cp.arg(container_path);
    // docker_cp.arg(&target_dir);
    //
    // tracing::trace!(?docker_cp, "Running docker command");
    //
    // eprintln!("CP Status: {:?}", docker_cp.status()?);

    Ok(())
}

fn checkout_project(project: &Component, src_dir: impl AsRef<Path>) -> anyhow::Result<()> {
    let url = &project
        .external_references
        .as_ref()
        .and_then(|v| v.get(0))
        .ok_or(anyhow::format_err!("Failed to get url for component: {}", project.name))?
        .url;
    let commit = &project
        .properties
        .as_ref()
        .and_then(|properties| properties.iter().find(|property| property.name == "commit"))
        .ok_or(anyhow::format_err!("Failed to get commit for component: {}", project.name))?
        .value;

    tracing::trace!("Clone repo {url} to {:?}", src_dir.as_ref());
    if !Command::new("git")
        .arg("clone")
        .arg(url)
        .arg(src_dir.as_ref())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .status()?
        .success()
    {
        anyhow::bail!("Failed to clone repo: {url}");
    }

    // checkout status is usually not success but it works
    // TODO: check status
    Command::new("git")
        .arg("checkout")
        .arg(commit)
        .current_dir(src_dir.as_ref())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .status()?;

    if let Some(hashes) = &project.hashes {
        let git_archive_data = Command::new("git")
            .arg("archive")
            .arg("--format=tar")
            .arg(commit)
            .current_dir(src_dir.as_ref())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?
            .stdout;

        check_hashes(hashes, git_archive_data)?;
    }

    Ok(())
}

fn checkout_dependencies(sbom: &CycloneDXBom, deps_dir: impl AsRef<Path>) -> anyhow::Result<()> {
    tracing::trace!("Load cargo dependencies to {:?}", deps_dir.as_ref());
    load_dependencies(sbom, deps_dir)
}
