use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VersionType {
    Vanilla,
    NeoForge,
}

#[derive(Debug, Deserialize)]
pub struct VersionManifestIndex {
    pub versions: Vec<VersionEntry>,
}

#[derive(Debug, Deserialize)]
pub struct VersionEntry {
    pub id: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionManifest {
    pub id: String,
    pub main_class: String,
    pub arguments: Option<Arguments>,
    pub libraries: Vec<Library>,
    pub downloads: Option<Downloads>,
    pub asset_index: Option<AssetIndex>,
    pub inherits_from: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Arguments {
    pub game: Option<Vec<serde_json::Value>>,
    pub jvm: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
pub struct Library {
    pub name: String,
    pub downloads: Option<LibraryDownloads>,
    pub rules: Option<Vec<Rule>>,
    pub natives: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct LibraryDownloads {
    pub artifact: Option<Artifact>,
    pub classifiers: Option<HashMap<String, Artifact>>,
}

#[derive(Debug, Deserialize)]
pub struct Artifact {
    pub url: String,
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct Rule {
    pub action: String,
    pub os: Option<OsRule>,
}

#[derive(Debug, Deserialize)]
pub struct OsRule {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Downloads {
    pub client: ClientDownload,
}

#[derive(Debug, Deserialize)]
pub struct ClientDownload {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct AssetIndex {
    pub url: String,
}
