mod constants;

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Once;

use anytree_sbom::Component;

use crate::cargo_components::git::constants::*;
use crate::cargo_components::helper::{get_component_properties, get_suffix_hash};
use crate::crypto::hash::check_hashes;

static INIT: Once = Once::new();
pub const LIBRARY_TYPE: &str = "cargo/git";

pub struct CargoGitComponent {}

fn init(cargo_root: impl AsRef<Path>) {
    tracing::trace!("Init cargo git dirs.");
    // create default cache dir tag
    let mut git_path = PathBuf::from(cargo_root.as_ref());
    git_path.push(CARGO_GIT_SUBFOLDER);
    std::fs::create_dir_all(&git_path).expect("Failed to create directory for cargo git registry");
    git_path.push(CACHE_DIR_TAG_NAME);
    let mut cache_dir_tag_file =
        File::create(git_path.to_str().unwrap()).expect("Failed to create default cargo git tag");
    cache_dir_tag_file
        .write_all(DEFAULT_CACHE_DIR_TAG.as_bytes())
        .expect("Failed to write default cargo git tag");
}

impl CargoGitComponent {
    pub fn save(cargo_root: &Path, component: &Component) -> anyhow::Result<()> {
        tracing::trace!("Start save of cargo git component {}", component.name,);
        INIT.call_once(|| init(cargo_root));
        let mut path = PathBuf::from(cargo_root);
        path.push(CARGO_GIT_SUBFOLDER);

        let name = &component.name;
        let properties = get_component_properties(component)?;
        let url = &component
            .external_references
            .as_ref()
            .and_then(|v| v.get(0))
            .ok_or(anyhow::format_err!("Failed to get url for component: {}", component.name))?
            .url;
        let commit = properties
            .get("commit")
            .ok_or(anyhow::format_err!("Failed to get dependency commit"))?;

        let mut clone_dir = path.clone();
        clone_dir.push(DB_SUBFOLDER);

        let dir_suffix = get_suffix_hash(url, None);
        clone_dir.push(format!("{}-{}", name, &dir_suffix));

        // Clone bare repo
        tracing::trace!("Cloning the bare repo. url: {}", &url);
        let status = Command::new("git")
            .arg("clone")
            .arg("--bare")
            .arg(url)
            .arg(clone_dir.as_os_str())
            .stderr(Stdio::piped())
            .status()?;

        if !status.success() {
            anyhow::bail!("Failed to clone bare repo: {}", url);
        }

        // check hashes if specified in SBOM
        if let Some(hashes) = &component.hashes {
            let data = git_archive(&clone_dir, commit)?;
            check_hashes(hashes, data)?;
        }

        // Simple bare clone is not enough for cargo install need to stare ref
        let mut ref_path = clone_dir.clone();
        ref_path.push(REF_PATH);
        std::fs::create_dir_all(&ref_path)?;
        ref_path.push(REF_FILE_NAME);
        if !ref_path.exists() {
            let mut file = File::create(&ref_path)?;
            file.write_all(commit.clone().as_bytes())?;
        }

        let mut checkout_dir = path.clone();
        checkout_dir.push(CHECKOUTS_SUBFOLDER);
        checkout_dir.push(format!("{}-{}", name, &dir_suffix));
        let mut trimmed_commit = commit.clone();
        trimmed_commit.truncate(7);
        checkout_dir.push(trimmed_commit);
        std::fs::create_dir_all(&checkout_dir)?;

        // clone dir from bare repo
        tracing::trace!("Cloning from bare repo to the ordinary one. path: {:?}", &checkout_dir);
        let status = Command::new("git")
            .arg("clone")
            .arg(clone_dir.as_os_str())
            .arg(checkout_dir.as_os_str())
            .stderr(Stdio::piped())
            .status()?;

        if !status.success() {
            anyhow::bail!("Failed to clone repo: {:?}", clone_dir);
        }

        // checkout commit
        tracing::trace!("Checkout the commit dir: {:?}, commit: {}", &checkout_dir, commit);
        let status = Command::new("git")
            .arg("checkout")
            .arg(commit)
            .current_dir(checkout_dir.as_os_str())
            .stderr(Stdio::piped())
            .status()?;

        if !status.success() {
            anyhow::bail!("Failed to checkout commit: {} in {:?}", commit, checkout_dir);
        }

        checkout_dir.push(CARGO_OK_FILE_NAME);
        if !checkout_dir.exists() {
            tracing::trace!("Create a cargo-ok file: {:?}", &checkout_dir);
            File::create(&checkout_dir)?;
        }

        Ok(())
    }
}

pub fn git_archive(repo: impl AsRef<Path>, commit: impl AsRef<str>) -> anyhow::Result<Vec<u8>> {
    let git_archive_output = Command::new("git")
        .arg("archive")
        .arg("--format=tar")
        .arg(commit.as_ref())
        .current_dir(repo.as_ref())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    Ok(git_archive_output.stdout)
}
