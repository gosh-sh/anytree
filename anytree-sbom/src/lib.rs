//! See https://github.com/CycloneDX

use chrono::offset::Utc;
use chrono::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CycloneDXBom {
    #[serde(rename = "bomFormat")]
    pub bom_format: String,
    #[serde(rename = "specVersion")]
    pub spec_version: String,
    #[serde(rename = "serialNumber")]
    pub serial_number: Option<String>,
    pub version: i32,
    pub metadata: Option<Metadata>,
    pub components: Vec<Component>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Metadata {
    pub timestamp: DateTime<Utc>,
    pub tools: Vec<Tool>,
    pub component: Component,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Tool {
    pub vendor: String,
    pub name: String,
    pub version: String,
    pub hashes: Vec<Hash>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Hash {
    pub alg: String,
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Component {
    pub bom_ref: Option<String>,
    #[serde(rename = "type")]
    pub component_type: String,
    pub name: String,
    pub version: String,
    pub purl: String,
    #[serde(rename = "externalReferences")]
    pub external_references: Option<Vec<ExternalReference>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExternalReference {
    pub url: String,
    #[serde(rename = "type")]
    pub ref_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bom() {
        let sbom_file = include_str!("../tests/fixtures/proton-bridge-v1.6.3.bom.json");
        let sbom: CycloneDXBom = serde_json::from_reader(sbom_file.as_bytes()).unwrap();

        assert_eq!(sbom.bom_format, "CycloneDX");
        assert_eq!(sbom.components.first().unwrap().external_references.as_ref().unwrap().len(), 1);
    }
}
