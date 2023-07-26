mod constants;

use crate::cargo_components::helper::{
    convert_index_to_cache, get_component_properties, name_to_index_path,
};
use crate::cargo_components::registry::constants::*;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Once;
use anytree_sbom::Component;

static INIT: Once = Once::new();
pub const LIBRARY_TYPE: &str = "rust/registry";

pub struct CargoRegistryComponent {}

fn init(cargo_cache_path: impl AsRef<Path>) {
    // create default crates.io config
    let mut index_path = PathBuf::from(cargo_cache_path.as_ref());
    index_path.push(CARGO_REGISTRY_SUBFOLDER);
    index_path.push(REGISTRY_INDEX_CACHE_PREFIX);
    std::fs::create_dir_all(&index_path)
        .expect("Failed to create directory for cargo crates registry");
    let mut index_config_path = index_path.clone();
    index_config_path.pop();
    index_config_path.push(INDEX_CONFIG_NAME);
    let mut config_file = File::create(index_config_path.to_str().unwrap())
        .expect("Failed to create default cargo registry config");
    config_file
        .write(DEFAULT_INDEX_CONFIG.as_bytes())
        .expect("Failed to write default cargo registry config");
}

impl CargoRegistryComponent {
    pub fn save(
        cargo_cache_path: &Path,
        bom_component: &Component,
    ) -> anyhow::Result<()> {
        INIT.call_once(|| init(cargo_cache_path));
        let mut path = PathBuf::from(cargo_cache_path);
        path.push(CARGO_REGISTRY_SUBFOLDER);

        let name = format!(
            "{}-{}",
            bom_component.name.to_string(),
            bom_component.version.to_string()
        );
        let properties = get_component_properties(bom_component)?;
        let url = properties
            .get("url")
            .ok_or(anyhow::format_err!("Failed to get dependency URL"))?;
        let commit = properties
            .get("commit")
            .ok_or(anyhow::format_err!("Failed to get dependency commit"))?;

        // prepare dir for dependency source files
        let mut src_path = path.clone();
        src_path.push(REGISTRY_SRC_PREFIX);
        src_path.push(&name);
        std::fs::create_dir_all(&src_path)?;

        // load dependency as archive with specified commit
        let mut cache_path = path.clone();
        cache_path.push(REGISTRY_CACHE_PREFIX);
        std::fs::create_dir_all(&cache_path)?;
        let cache_dir = cache_path.to_str().unwrap();
        let cache_name = format!("{name}.crate");


        // TODO: replace registry
        // registry.dump_commit(url, commit, &src_path).await?;


        // convert index to cargo cache and save to file
        let index_str = properties
            .get("crates_io_index")
            .ok_or(anyhow::format_err!("Failed to get dependency crate index"))?;
        let mut index_path = path.clone();
        index_path.push(REGISTRY_INDEX_CACHE_PREFIX);
        index_path.push(name_to_index_path(&bom_component.name.to_string()));
        let mut index_dir = index_path.clone();
        index_dir.pop();
        std::fs::create_dir_all(index_dir)?;
        convert_index_to_cache(index_str, index_path)?;

        // Cargo writes .cargo-ok file into src dir but mount in Dockerfile in read only
        // so we create this file if it doesn't exist
        let mut cargo_ok_path = src_path.clone();
        cargo_ok_path.push(".cargo-ok");

        if !cargo_ok_path.exists() {
            let mut file = File::create(&cargo_ok_path)?;
            file.write("ok".as_bytes())?;
        }

        // archive the source directory
        let mut src_dir = src_path.clone();
        src_dir.pop();
        let status = Command::new("tar")
            .arg("-zcf")
            .arg(cache_name)
            .arg("--directory")
            .arg(src_dir)
            .arg(&name)
            .current_dir(cache_dir)
            .status()?;
        if !status.success() {
            anyhow::bail!("Failed to compress sources: {}", status);
        }
        Ok(())
    }
}
