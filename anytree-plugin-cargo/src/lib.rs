use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::vec;

use anytree_plugin_cargo_dependencies::crypto::hash::check_hashes;
use anytree_plugin_cargo_dependencies::load_dependencies;
use anytree_sbom::{Component, CycloneDXBom};
use anytree_utils::tracing::wrap_cmd_with_tracing;

const PROJECT_DIR: &str = "src";
const DEPENDENCIES_DIR: &str = "cargo";
const TARGET_DIR: &str = "target";

const CONTAINER_PROJECT_DIR: &str = "/tmp/proj/";
const CONTAINER_REGISTRY_ROOT: &str = "/usr/local/cargo/";

const CONTAINER_BASE: &str = "rust:1.71";

pub const PROJECT_TYPE: &str = "cargo/project";

pub fn build(
    project: &Component,
    sbom: &CycloneDXBom,
    container_name: &str,
    run_dir: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let src_dir = run_dir.as_ref().join(PROJECT_DIR);
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

    tracing::trace!("Prepare build container");

    let mut docker_cmd = Command::new("docker");
    docker_cmd.arg("run");
    docker_cmd.arg("-t");
    docker_cmd.arg("--network").arg("none");
    docker_cmd.arg("--name").arg(container_name);

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

    docker_cmd.arg("--workdir").arg(CONTAINER_PROJECT_DIR);
    docker_cmd.arg(CONTAINER_BASE);
    docker_cmd.arg("sh").arg("-c");

    // By default build with standard command.
    // TODO: add ability to specify build command in config
    let build_cmd = "cargo build --offline --release".to_string();
    docker_cmd.arg(build_cmd);

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

    let artifact_name = &project
        .properties
        .as_ref()
        .and_then(|properties| properties.iter().find(|property| property.name == "result"))
        .ok_or(anyhow::format_err!("Failed to get artifact name for component: {}", project.name))?
        .value;

    let mut docker_cp = Command::new("docker");
    docker_cp.arg("cp");

    let mut container_path = OsString::from(container_name);
    container_path.push(":");
    container_path.push(CONTAINER_PROJECT_DIR);
    container_path.push("/target/release/");
    container_path.push(artifact_name);

    tracing::trace!("Container artifact path: {:?}", container_path);

    docker_cp.arg(container_path);
    docker_cp.arg(&target_dir);
    tracing::trace!(?docker_cp, "Running docker command");

    let res = docker_cp.status()?;
    if !res.success() {
        anyhow::bail!("docker command failed: {}", res.code().unwrap_or(-1));
    }

    Ok(())
}

fn checkout_project(project: &Component, src_dir: impl AsRef<Path>) -> anyhow::Result<()> {
    tracing::trace!("Load project to {:?}", src_dir.as_ref());
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

    tracing::info!("Checking out project {url}#{commit}");

    // Clone the repo
    if !Command::new("git")
        .arg("clone")
        .arg(url)
        .arg(src_dir.as_ref())
        // inherit all output because it's sort of user output via our bin
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?
        .success()
    {
        anyhow::bail!("Failed to clone repo: {url}");
    }

    // checkout commit specified in SBOM, do not delete
    // TODO: check status
    Command::new("git")
        .arg("checkout")
        .arg(commit)
        .current_dir(src_dir.as_ref())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .status()?;

    if let Some(hashes) = &project.hashes {
        // To check hash get archive bytes of the repo
        let git_archive_data = Command::new("git")
            .arg("archive")
            .arg("--format=tar")
            .arg(commit)
            .current_dir(src_dir.as_ref())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?
            .stdout;

        tracing::info!("Checking project hashes");
        check_hashes(hashes, git_archive_data)?;
    }

    Ok(())
}

fn checkout_dependencies(sbom: &CycloneDXBom, deps_dir: impl AsRef<Path>) -> anyhow::Result<()> {
    tracing::trace!("Load cargo dependencies to {:?}", deps_dir.as_ref());
    load_dependencies(sbom, deps_dir)
}
