use std::collections::HashMap;
use std::fs::File;
use std::hash::{Hash, Hasher, SipHasher};
use std::io::Write;
use std::path::Path;

use anytree_sbom::Component;

/// https://doc.rust-lang.org/cargo/reference/registry-index.html#index-files
/// Packages with 1 character names are placed in a directory named 1.
/// Packages with 2 character names are placed in a directory named 2.
/// Packages with 3 character names are placed in the directory
///   3/{first-character} where {first-character} is the first character of the
///   package name.
/// All other packages are stored in directories named
///   {first-two}/{second-two} where the top directory is the first two
///   characters of the package name, and the next subdirectory is the third and
///   fourth characters of the package name. For example, cargo would be stored
///   in a file named ca/rg/cargo.
pub fn name_to_index_path(crate_name: &str) -> String {
    match crate_name.len() {
        0 => panic!("crate name can't be empty"),
        1 => format!("1/{}", crate_name),
        2 => format!("2/{}", crate_name),
        3 => {
            let mut first_char = crate_name.clone().to_string();
            first_char.truncate(1);
            format!("3/{}/{}", first_char, crate_name)
        }
        _ => {
            let mut header_iter =
                crate_name.clone().as_bytes().chunks(2).map(String::from_utf8_lossy);
            format!(
                "{}/{}/{}",
                header_iter.next().unwrap(),
                header_iter.next().unwrap(),
                crate_name
            )
        }
    }
}

// Cargo stores index in cache with special format
// https://github.com/rust-lang/cargo/blob/04c94d90b69617a1d744cc141deebea4ebdfd886/src/cargo/sources/registry/index.rs#L835
pub fn convert_index_to_cache(
    index_str: &str,
    output_path: impl AsRef<Path>,
) -> anyhow::Result<()> {
    // start with headers  [cache_version] [index_v_max]
    let mut res_bytes = vec![3_u8, 2_u8, 0, 0, 0];

    // than etag, which we do not generate but use a random value
    let etag = "etag: W/\"bbbf8771a5922743c5e0b466d90e7ab6\"";
    res_bytes.append(&mut etag.as_bytes().to_vec());
    res_bytes.push(0);

    for line in index_str.split("\n") {
        if line.is_empty() {
            break;
        }
        let object: serde_json::Value = serde_json::from_str(line)?;
        let version = object.as_object().unwrap()["vers"].as_str().unwrap();
        res_bytes.append(&mut version.as_bytes().to_vec());
        res_bytes.push(0);
        res_bytes.append(&mut line.as_bytes().to_vec());
        res_bytes.push(0);
    }

    // Save to the file
    let mut ofile = File::create(output_path)?;
    ofile.write_all(res_bytes.as_slice())?;
    Ok(())
}

pub fn get_component_properties(component: &Component) -> anyhow::Result<HashMap<String, String>> {
    // TODO: change to trait
    let mut result = HashMap::new();
    if let Some(properties) = &component.properties {
        for property in properties {
            result.insert(property.name.clone(), property.value.to_string());
        }
    }
    Ok(result)
}

/// Slightly trimmed struct from cargo
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SourceKind {
    /// A git repository.
    _Git, // This variant was simplified for not to add extra structs
    /// A local path.
    _Path,
    /// A remote registry.
    _Registry,
    /// A sparse registry.
    SparseRegistry,
    /// A local filesystem-based registry.
    _LocalRegistry,
    /// A directory-based registry.
    _Directory,
}

// The simplified mechanism how cargo obtains hash of the git url to calculate
// dir suffix
pub fn get_suffix_hash(url: &str, kind: Option<SourceKind>) -> String {
    // NOTE: SipHasher is deprecated, but cargo seems to check this hash !!!!
    let mut hasher = SipHasher::new();
    if let Some(kind) = kind {
        kind.hash(&mut hasher);
    }
    let url = url.trim_end_matches(".git").to_lowercase();
    url.hash(&mut hasher);
    let res: u64 = hasher.finish();
    hex::encode(res.to_le_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_to_index() {
        assert_eq!("1/a", name_to_index_path("a"));
        assert_eq!("2/aa", name_to_index_path("aa"));
        assert_eq!("3/r/ryu", name_to_index_path("ryu"));
        assert_eq!("se/rd/serde", name_to_index_path("serde"));
    }

    #[test]
    fn test_suffix() {
        assert_eq!(
            "1ecc6299db9ec823",
            get_suffix_hash(
                "https://github.com/rust-lang/crates.io-index",
                Some(SourceKind::_Registry)
            )
        );
        assert_eq!(
            "6f17d22bba15001f",
            get_suffix_hash("sparse+https://index.crates.io/", Some(SourceKind::SparseRegistry))
        );
        assert_eq!(
            "f9cb9f02e39b3874",
            get_suffix_hash("https://github.com/silkovalexander/simple_lib", None)
        );
    }
}
