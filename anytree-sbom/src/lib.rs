//! See https://github.com/CycloneDX

use chrono::offset::Utc;
use chrono::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CycloneDXBom {
    #[serde(rename = "bomFormat")]
    pub bom_format: String,
    #[serde(rename = "specVersion")]
    pub spec_version: String,
    #[serde(rename = "serialNumber")]
    pub serial_number: Option<String>,
    // add default
    pub version: i32,
    pub metadata: Option<Metadata>,
    pub components: Vec<Component>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Metadata {
    pub timestamp: Option<DateTime<Utc>>,
    pub tools: Option<Vec<Tool>>, // vec of tool is deprecated in 1.5
    pub component: Option<Component>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tool {
    pub vendor: Option<String>,
    pub name: Option<String>,
    pub version: Option<String>,
    pub hashes: Option<Vec<Hash>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Hash {
    pub alg: String,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Property {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComponentType {
    #[serde(rename = "application")]
    Application,
    #[serde(rename = "framework")]
    Framework,
    #[serde(rename = "library")]
    Library,
    #[serde(rename = "container")]
    Container,
    #[serde(rename = "operating-system")]
    OperatingSystem,
    #[serde(rename = "device")]
    Device,
    #[serde(rename = "firmware")]
    Firmware,
    #[serde(rename = "file")]
    File,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Component {
    pub bom_ref: Option<String>,
    #[serde(rename = "type")]
    pub component_type: ComponentType,
    pub name: String,
    pub version: String,
    pub purl: String,
    #[serde(rename = "externalReferences")]
    pub external_references: Option<Vec<ExternalReference>>,
    pub properties: Option<Vec<Property>>,
    #[serde(rename = "mime-type")]
    pub mime_type: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExternalReference {
    pub url: String,
    #[serde(rename = "type")]
    pub ref_type: String,
}

#[cfg(test)]
mod tests {
    use serde_json::error::Category;

    use super::*;

    #[test]
    fn test_optimistic_bom() {
        let json = include_bytes!("../tests/fixtures/proton-bridge-v1.6.3.cdx.json");
        let sbom = serde_json::from_slice::<CycloneDXBom>(json).unwrap();

        assert_eq!(sbom.bom_format, "CycloneDX");
        assert_eq!(sbom.components.first().unwrap().external_references.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_wrong_type_should_fail() {
        let json = include_bytes!("../tests/fixtures/wrong_component_type.cdx.json");

        let Err(err) = serde_json::from_slice::<CycloneDXBom>(json) else {
            panic!("Expected wrong type error")
        };

        assert!(err.is_data());
    }
}
