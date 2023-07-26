mod constants;

use crate::cargo_components::git::constants::*;
use crate::cargo_components::helper::get_component_properties;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Once;
use anytree_sbom::Component;

static INIT: Once = Once::new();
pub const LIBRARY_TYPE: &str = "rust/git";

pub struct CargoGitComponent {}

fn init(cargo_cache_path: impl AsRef<Path>) {
    // create default cache dir tag
    let mut git_path = PathBuf::from(cargo_cache_path.as_ref());
    git_path.push(CARGO_GIT_SUBFOLDER);
    std::fs::create_dir_all(&git_path).expect("Failed to create directory for cargo git registry");
    git_path.push(CACHE_DIR_TAG_NAME);
    let mut cache_dir_tag_file =
        File::create(git_path.to_str().unwrap()).expect("Failed to create default cargo git tag");
    cache_dir_tag_file
        .write(DEFAULT_CACHE_DIR_TAG.as_bytes())
        .expect("Failed to write default cargo git tag");
}


impl CargoGitComponent {
    pub fn save(
        cargo_cache_path: &Path,
        bom_component: &Component,
    ) -> anyhow::Result<()> {
        INIT.call_once(|| init(cargo_cache_path));
        let mut path = PathBuf::from(cargo_cache_path);
        path.push(CARGO_GIT_SUBFOLDER);

        let name = format!("{}", bom_component.name.to_string(),);
        let properties = get_component_properties(bom_component)?;
        let url = properties
            .get("url")
            .ok_or(anyhow::format_err!("Failed to get dependency URL"))?;
        let commit = properties
            .get("commit")
            .ok_or(anyhow::format_err!("Failed to get dependency commit"))?;

        let mut clone_dir = path.clone();
        clone_dir.push(DB_SUBFOLDER);
        clone_dir.push(format!("{}-{}", name, DIR_SUFFIX));


        // TODO: replace registry
        // let git_cache = GitCacheRepo::with_dir(url.to_string(), clone_dir.clone());
        // git_cache.bare_clone().await?;


        // Simple bare clone is not enough for cargo install need to stare ref
        clone_dir.push(REF_PATH);
        std::fs::create_dir_all(&clone_dir)?;
        clone_dir.push(REF_FILE_NAME);
        if !clone_dir.exists() {
            let mut file = File::create(&clone_dir)?;
            file.write(commit.clone().as_bytes())?;
        }

        let mut checkout_dir = path.clone();
        checkout_dir.push(CHECKOUTS_SUBFOLDER);
        checkout_dir.push(format!("{}-{}", name, DIR_SUFFIX));
        let mut trimmed_commit = commit.clone();
        trimmed_commit.truncate(7);
        checkout_dir.push(trimmed_commit);


        // TODO: replace registry
        // let git_cache = GitCacheRepo::with_dir(url.to_string(), checkout_dir.clone());
        // git_cache.update().await?;
        // git_cache.checkout(commit).await?;

        checkout_dir.push(".cargo-ok");
        if !checkout_dir.exists() {
            File::create(&checkout_dir)?;
        }

        Ok(())
    }
}
