mod constants;

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Once;

use anytree_sbom::Component;
use anytree_utils::crypto::hash::check_hashes;

use crate::cargo_components::helper::SourceKind::SparseRegistry;
use crate::cargo_components::helper::{
    convert_index_to_cache, get_suffix_hash, name_to_index_path,
};
use crate::cargo_components::registry::constants::*;

static INIT: Once = Once::new();
pub const LIBRARY_TYPE: &str = "cargo/registry";

pub struct CargoRegistryComponent {}

fn init(cargo_root: impl AsRef<Path>) {
    tracing::trace!("Init cargo registry dir");
    // create default crates.io config
    let mut index_path = PathBuf::from(cargo_root.as_ref());
    index_path.push(CARGO_REGISTRY_SUBFOLDER);
    index_path.push(CARGO_INDEX_SUBFOLDER);
    index_path.push(get_registry_path());
    index_path.push(CARGO_INDEX_CACHE_SUBFOLDER);
    std::fs::create_dir_all(&index_path)
        .expect("Failed to create directory for cargo crates registry");
    let mut index_config_path = index_path.clone();
    index_config_path.pop();
    index_config_path.push(INDEX_CONFIG_NAME);
    let mut config_file = File::create(index_config_path.to_str().unwrap())
        .expect("Failed to create default cargo registry config");
    config_file
        .write_all(DEFAULT_INDEX_CONFIG.as_bytes())
        .expect("Failed to write default cargo registry config");
}

fn get_registry_path() -> String {
    format!(
        "{}-{}",
        CARGO_REGISTRY_PREFIX,
        get_suffix_hash(CARGO_REGISTRY_URL, Some(SparseRegistry))
    )
}

impl CargoRegistryComponent {
    pub fn save(cargo_root: &Path, component: &Component) -> anyhow::Result<()> {
        let version = component
            .version
            .as_ref()
            .ok_or(anyhow::format_err!("Component {} does not contain version", component.name))?;
        tracing::info!("Loading cargo registry component {}.{}", component.name, version,);
        INIT.call_once(|| init(cargo_root));
        let mut path = PathBuf::from(cargo_root);
        path.push(CARGO_REGISTRY_SUBFOLDER);

        let name = format!("{}-{}", component.name, version);
        let url = &component
            .external_references
            .as_ref()
            .and_then(|ext_refs| ext_refs.get(0))
            .map(|reference| reference.url.as_str())
            .ok_or(anyhow::format_err!(
                "Component {} does not contain external references",
                component.name
            ))?;

        // load dependency as archive with specified commit
        let mut cache_path = path.clone();
        cache_path.push(CARGO_CACHE_SUBFOLDER);
        cache_path.push(get_registry_path());
        std::fs::create_dir_all(&cache_path)?;
        let cache_dir = cache_path.clone();
        let cache_dir = cache_dir.to_str().unwrap();
        let cache_name = format!("{name}.crate");
        cache_path.push(&cache_name);

        if cache_path.exists() {
            return Ok(());
        }

        // Download crate as archive
        tracing::trace!("Downloading crate as an archive. url: {}", &url);
        let status = Command::new("curl")
            .arg("-L")
            .arg(url)
            .arg("--output")
            .arg(cache_name)
            .current_dir(cache_dir)
            .stderr(Stdio::piped())
            .status()?;
        if !status.success() {
            anyhow::bail!("Failed to download archive: {}", url);
        }

        if let Some(hashes) = &component.hashes {
            let data = std::fs::read(&cache_path)?;
            tracing::info!("Check hash for {}", &name);
            check_hashes(hashes, data)?;
        }

        // prepare dir for dependency source files
        let mut src_path = path.clone();
        src_path.push(CARGO_SRC_SUBFOLDER);
        src_path.push(get_registry_path());
        std::fs::create_dir_all(&src_path)?;

        // tar -xzf itoa-1.0.8.crate -o itoa-1.0.8
        // extract the source directory
        tracing::trace!("Extracting the crate archive.");
        let mut src_dir = src_path.clone();
        src_dir.pop();
        let status = Command::new("tar")
            .arg("-xzf")
            .arg(cache_path)
            .arg("-o")
            .arg(&name)
            .current_dir(&src_path)
            .status()?;
        if !status.success() {
            anyhow::bail!("Failed to compress sources: {}", status);
        }

        // Prepare index file
        let mut index_path = path.clone();
        index_path.push(CARGO_INDEX_SUBFOLDER);
        index_path.push(get_registry_path());
        index_path.push(CARGO_INDEX_CACHE_SUBFOLDER);
        index_path.push(name_to_index_path(&component.name.to_string()));
        let mut index_dir = index_path.clone();
        index_dir.pop();
        std::fs::create_dir_all(index_dir)?;
        let indexed_name = name_to_index_path(&component.name);
        let index_url = format!("{}{}", CRATE_INDEX_URL, indexed_name,);
        // Download index file
        tracing::trace!("Downloading the index. url: {}", index_url);
        let status =
            Command::new("curl").arg("-L").arg(&index_url).stderr(Stdio::piped()).output()?;
        if !status.status.success() {
            anyhow::bail!("Failed to download crate index: {}", index_url);
        }
        // let mut index_str = None;
        let lines = std::str::from_utf8(status.stdout.as_slice())?;
        // for line in lines.split('\n') {
        //     if line.contains(version) {
        //         index_str = Some(line.to_string());
        //     }
        // }
        // let index_str = match index_str {
        //     Some(s) => s,
        //     None => {
        //         anyhow::bail!("Failed to get index for specified version");
        //     }
        // };
        // convert index to cargo cache and save to file
        convert_index_to_cache(lines, index_path)?;

        // Cargo writes .cargo-ok file into src dir but mount in Dockerfile in read only
        // so we create this file if it doesn't exist
        let mut cargo_ok_path = src_path.clone();
        cargo_ok_path.push(&name);
        cargo_ok_path.push(CARGO_OK_FILE_NAME);

        if !cargo_ok_path.exists() {
            tracing::trace!("Adding cargo-ok to {:?}.", cargo_ok_path);
            let mut file = File::create(&cargo_ok_path)?;
            file.write_all(CARGO_OK_CONTENT.as_bytes())?;
        }

        Ok(())
    }
}
