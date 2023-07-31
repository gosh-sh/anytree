pub const CARGO_REGISTRY_URL: &str = "sparse+https://index.crates.io/";
pub const CARGO_REGISTRY_PREFIX: &str = "index.crates.io";
pub const CRATE_INDEX_URL: &str = "https://github.com/rust-lang/crates.io-index/raw/master/";

pub const CARGO_REGISTRY_SUBFOLDER: &str = "registry";
pub const CARGO_INDEX_SUBFOLDER: &str = "index";
pub const CARGO_INDEX_CACHE_SUBFOLDER: &str = ".cache";
pub const CARGO_CACHE_SUBFOLDER: &str = "cache";
pub const CARGO_SRC_SUBFOLDER: &str = "src";

pub const INDEX_CONFIG_NAME: &str = "config.json";
pub const CARGO_OK_FILE_NAME: &str = ".cargo-ok";

pub const DEFAULT_INDEX_CONFIG: &str = "{
  \"dl\": \"https://crates.io/api/v1/crates\",
  \"api\": \"https://crates.io\"
}";
pub const CARGO_OK_CONTENT: &str = "ok";
