use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use anytree_sbom::Component;

// https://doc.rust-lang.org/cargo/reference/registry-index.html#index-files
// Packages with 1 character names are placed in a directory named 1.
// Packages with 2 character names are placed in a directory named 2.
// Packages with 3 character names are placed in the directory 3/{first-character} where
//   {first-character} is the first character of the package name.
// All other packages are stored in directories named {first-two}/{second-two} where the top directory
//   is the first two characters of the package name, and the next subdirectory is the third and fourth
//   characters of the package name. For example, cargo would be stored in a file named ca/rg/cargo.
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

    // in real cache then go list of all versions followed by this version index
    // but we store only one version we use with index obtained from SBOM
    let object: serde_json::Value = serde_json::from_str(index_str)?;
    let version = object.as_object().unwrap()["vers"].as_str().unwrap();
    res_bytes.append(&mut version.as_bytes().to_vec());
    res_bytes.push(0);
    res_bytes.append(&mut index_str.as_bytes().to_vec());
    res_bytes.push(0);

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
}
