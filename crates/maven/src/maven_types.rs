use serde::{Deserialize, Serialize};
use serde_xml_rs::from_reader;
use std::io::Read;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub group_id: String,
    pub artifact_id: String,
    pub versioning: Versioning,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Versioning {
    pub latest: String,
    pub release: String,
    pub versions: Versions,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Versions {
    pub version: Vec<String>,
}

impl Metadata {
    pub fn from_xml<R: Read>(reader: R) -> Metadata {
        from_reader(reader).expect("Could not parse metadata xml")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_xml_rs::from_reader;
    use std::fs::read_to_string;
    use std::path::PathBuf;

    #[test]
    fn parse() {
        let file =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/resources/example_metadata.xml");
        let content = read_to_string(file).expect("Could not read file");

        let target: Metadata = from_reader(content.as_bytes()).expect("could not parse xml");
        let expected = Metadata {
            group_id: "de.kinch".to_string(),
            artifact_id: "my-artifact".to_string(),
            versioning: Versioning {
                latest: "2.4.3".to_string(),
                release: "2.4.3".to_string(),
                versions: Versions {
                    version: vec![
                        "2.0.0".to_string(),
                        "2.1.0".to_string(),
                        "2.2.0".to_string(),
                    ],
                },
            },
        };

        assert_eq!(target, expected);
    }
}
