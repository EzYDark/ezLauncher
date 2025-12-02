use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

const VERSION_MANIFEST_URL: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
const MC_VERSION: &str = "1.21.1";
const AUTHLIB_INJECTOR_URL: &str = "https://github.com/yushijinhun/authlib-injector/releases/download/v1.2.5/authlib-injector-1.2.5.jar";
const ELY_BY_API: &str = "https://authserver.ely.by/api/authlib-injector";

// ============================================================================
// Structures matching Mojang's JSON format
// ============================================================================

#[derive(Debug, Deserialize)]
struct VersionManifestIndex {
    versions: Vec<VersionEntry>,
}

#[derive(Debug, Deserialize)]
struct VersionEntry {
    id: String,
    url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VersionManifest {
    id: String,
    main_class: String,
    arguments: Option<Arguments>,
    libraries: Vec<Library>,
    downloads: Option<Downloads>,
    asset_index: Option<AssetIndex>,
}

#[derive(Debug, Deserialize)]
struct Arguments {
    game: Option<Vec<serde_json::Value>>,
    jvm: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
struct Library {
    name: String,
    downloads: Option<LibraryDownloads>,
    rules: Option<Vec<Rule>>,
    natives: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
struct LibraryDownloads {
    artifact: Option<Artifact>,
    classifiers: Option<HashMap<String, Artifact>>,
}

#[derive(Debug, Deserialize)]
struct Artifact {
    url: String,
    path: String,
}

#[derive(Debug, Deserialize)]
struct Rule {
    action: String,
    os: Option<OsRule>,
}

#[derive(Debug, Deserialize)]
struct OsRule {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Downloads {
    client: ClientDownload,
}

#[derive(Debug, Deserialize)]
struct ClientDownload {
    url: String,
}

#[derive(Debug, Deserialize)]
struct AssetIndex {
    url: String,
}

// ============================================================================
// Java Installation (reusing existing Adoptium logic)
// ============================================================================

#[derive(Debug, Deserialize)]
struct AdoptiumRelease {
    binary: AdoptiumBinary,
}

#[derive(Debug, Deserialize)]
struct AdoptiumBinary {
    package: AdoptiumPackage,
}

#[derive(Debug, Deserialize)]
struct AdoptiumPackage {
    link: String,
}

async fn install_java(base_dir: &Path) -> Result<PathBuf> {
    let java_dir = base_dir.join("java");
    let jdk_dir = java_dir.join("jdk-21.0.9+10");
    let java_bin = jdk_dir.join("bin").join("java");

    if java_bin.exists() {
        log::info!("Java already installed");
        return Ok(java_bin);
    }

    log::info!("Downloading Java 21...");
    let client = Client::builder()
        .user_agent("ezLauncher/0.2.0")
        .build()?;

    let url = "https://api.adoptium.net/v3/assets/latest/21/hotspot?architecture=x64&image_type=jdk&os=linux&vendor=eclipse";
    let releases: Vec<AdoptiumRelease> = client.get(url).send().await?.json().await?;

    let release = releases
        .first()
        .ok_or_else(|| anyhow::anyhow!("No Java releases found"))?;

    let tar_path = java_dir.join("jdk.tar.gz");
    download_file(&client, &release.binary.package.link, &tar_path).await?;

    log::info!("Extracting Java...");
    tokio::fs::create_dir_all(&java_dir).await?;
    tokio::process::Command::new("tar")
        .arg("-xzf")
        .arg(&tar_path)
        .arg("-C")
        .arg(&java_dir)
        .status()
        .await?;

    tokio::fs::remove_file(tar_path).await?;
    Ok(java_bin)
}

// ============================================================================
// Minecraft Installation
// ============================================================================

async fn install_minecraft(base_dir: &Path) -> Result<(PathBuf, VersionManifest)> {
    let mc_dir = base_dir.join("minecraft");
    let version_dir = mc_dir.join("versions").join(MC_VERSION);
    let version_json_path = version_dir.join(format!("{}.json", MC_VERSION));
    let client_jar_path = version_dir.join(format!("{}.jar", MC_VERSION));

    let client = Client::builder()
        .user_agent("ezLauncher/0.2.0")
        .build()?;

    // Step 1: Get version manifest index
    log::info!("Fetching version manifest...");
    let manifest_index: VersionManifestIndex = client
        .get(VERSION_MANIFEST_URL)
        .send()
        .await?
        .json()
        .await?;

    // Step 2: Find our version
    let version_entry = manifest_index
        .versions
        .iter()
        .find(|v| v.id == MC_VERSION)
        .ok_or_else(|| anyhow::anyhow!("Version {} not found", MC_VERSION))?;

    // Step 3: Download version-specific JSON
    log::info!("Downloading version manifest for {}...", MC_VERSION);
    let version_json: String = client.get(&version_entry.url).send().await?.text().await?;
    let manifest: VersionManifest = serde_json::from_str(&version_json)?;

    // Save version JSON
    tokio::fs::create_dir_all(&version_dir).await?;
    tokio::fs::write(&version_json_path, &version_json).await?;

    // Step 4: Download client JAR
    if !client_jar_path.exists() {
        if let Some(downloads) = &manifest.downloads {
            log::info!("Downloading client JAR...");
            download_file(&client, &downloads.client.url, &client_jar_path).await?;
        }
    }

    // Step 5: Download libraries
    let lib_dir = mc_dir.join("libraries");
    for library in &manifest.libraries {
        // Check OS rules
        if let Some(rules) = &library.rules {
            if !check_rules(rules) {
                continue;
            }
        }

        if let Some(downloads) = &library.downloads {
            // Download main artifact
            if let Some(artifact) = &downloads.artifact {
                let lib_path = lib_dir.join(&artifact.path);
                if !lib_path.exists() {
                    log::info!("Downloading library: {}", library.name);
                    download_file(&client, &artifact.url, &lib_path).await?;
                }
            }

            // Download natives if present
            if let Some(natives) = &library.natives {
                if let Some(linux_classifier) = natives.get("linux") {
                    if let Some(classifiers) = &downloads.classifiers {
                        if let Some(native_artifact) = classifiers.get(linux_classifier) {
                            let native_path = lib_dir.join(&native_artifact.path);
                            if !native_path.exists() {
                                log::info!("Downloading native: {}", library.name);
                                download_file(&client, &native_artifact.url, &native_path).await?;
                            }
                        }
                    }
                }
            }
        }
    }

    // Step 6: Extract natives
    let natives_dir = mc_dir.join("natives");
    tokio::fs::create_dir_all(&natives_dir).await?;

    for library in &manifest.libraries {
        if let Some(natives) = &library.natives {
            if let Some(linux_classifier) = natives.get("linux") {
                if let Some(downloads) = &library.downloads {
                    if let Some(classifiers) = &downloads.classifiers {
                        if let Some(native_artifact) = classifiers.get(linux_classifier) {
                            let native_jar = lib_dir.join(&native_artifact.path);
                            if native_jar.exists() {
                                extract_natives(&native_jar, &natives_dir)?;
                            }
                        }
                    }
                }
            }
        }
    }

    // Step 7: Download authlib-injector for Ely.by authentication
    let authlib_path = base_dir.join("authlib-injector.jar");
    if !authlib_path.exists() {
        log::info!("Downloading authlib-injector...");
        download_file(&client, AUTHLIB_INJECTOR_URL, &authlib_path).await?;
    }

    Ok((mc_dir, manifest))
}

fn check_rules(rules: &[Rule]) -> bool {
    let mut allowed = false;
    for rule in rules {
        if rule.action == "allow" {
            if let Some(os) = &rule.os {
                if let Some(name) = &os.name {
                    if name == "linux" {
                        allowed = true;
                    }
                } else {
                    allowed = true; // No OS specified = allow all
                }
            } else {
                allowed = true; // No OS condition = allow all
            }
        } else if rule.action == "disallow" {
            if let Some(os) = &rule.os {
                if let Some(name) = &os.name {
                    if name == "linux" {
                        allowed = false;
                    }
                }
            }
        }
    }
    allowed
}

fn extract_natives(jar_path: &Path, natives_dir: &Path) -> Result<()> {
    let file = std::fs::File::open(jar_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_owned();

        // Skip META-INF and non-.so files
        if name.starts_with("META-INF") || file.is_dir() || !name.ends_with(".so") {
            continue;
        }

        let out_path = natives_dir.join(&name);
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut outfile = std::fs::File::create(&out_path)?;
        std::io::copy(&mut file, &mut outfile)?;
    }

    Ok(())
}

// ============================================================================
// Launch
// ============================================================================

async fn launch_game(
    mc_dir: PathBuf,
    java_path: PathBuf,
    manifest: VersionManifest,
    username: String,
    uuid: String,
    token: String,
) -> Result<()> {
    log::info!("Building launch command...");

    // Build classpath (all paths must be absolute)
    let lib_dir = mc_dir.join("libraries");
    let mut classpath = Vec::new();

    for library in &manifest.libraries {
        if let Some(rules) = &library.rules {
            if !check_rules(rules) {
                continue;
            }
        }

        if let Some(downloads) = &library.downloads {
            if let Some(artifact) = &downloads.artifact {
                let lib_path = lib_dir.join(&artifact.path);
                if lib_path.exists() {
                    if let Ok(absolute) = std::fs::canonicalize(&lib_path) {
                        classpath.push(absolute.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    // Add client JAR (absolute path)
    let client_jar = mc_dir
        .join("versions")
        .join(MC_VERSION)
        .join(format!("{}.jar", MC_VERSION));
    if let Ok(absolute) = std::fs::canonicalize(&client_jar) {
        classpath.push(absolute.to_string_lossy().to_string());
    }

    let classpath_str = classpath.join(":");

    // Build command
    let java_absolute = std::fs::canonicalize(&java_path)?;
    let mc_absolute = std::fs::canonicalize(&mc_dir)?;
    let natives_absolute = std::fs::canonicalize(mc_dir.join("natives"))?;
    
    let mut cmd = tokio::process::Command::new(&java_absolute);
    cmd.current_dir(&mc_absolute);

    // Memory args
    cmd.arg("-Xmx4G").arg("-Xms1G");

    // Authlib-injector for Ely.by authentication
    let authlib_path = mc_dir.parent()
        .ok_or_else(|| anyhow::anyhow!("Invalid mc_dir"))?
        .join("authlib-injector.jar");
    if let Ok(authlib_absolute) = std::fs::canonicalize(&authlib_path) {
        cmd.arg(format!(
            "-javaagent:{}={}",
            authlib_absolute.to_string_lossy(),
            ELY_BY_API
        ));
    }

    // Native library path
    cmd.arg(format!(
        "-Djava.library.path={}",
        natives_absolute.to_string_lossy()
    ));

    // Classpath
    cmd.arg("-cp").arg(&classpath_str);

    // JVM arguments from manifest (with variable substitution)
    if let Some(args) = &manifest.arguments {
        if let Some(jvm_args) = &args.jvm {
            for arg in jvm_args {
                if let serde_json::Value::String(s) = arg {
                    let substituted = s
                        .replace("${natives_directory}", &natives_absolute.to_string_lossy())
                        .replace("${launcher_name}", "ezLauncher")
                        .replace("${launcher_version}", "0.2.0")
                        .replace("${classpath}", &classpath_str);
                    cmd.arg(substituted);
                }
            }
        }
    }

    // Main class
    cmd.arg(&manifest.main_class);

    // Game arguments (with variable substitution)
    if let Some(args) = &manifest.arguments {
        if let Some(game_args) = &args.game {
            for arg in game_args {
                if let serde_json::Value::String(s) = arg {
                    let substituted = s
                        .replace("${auth_player_name}", &username)
                        .replace("${version_name}", MC_VERSION)
                        .replace("${game_directory}", &mc_absolute.to_string_lossy())
                        .replace("${assets_root}", &mc_absolute.join("assets").to_string_lossy())
                        .replace("${assets_index_name}", MC_VERSION)
                        .replace("${auth_uuid}", &uuid)
                        .replace("${auth_access_token}", &token)
                        .replace("${user_type}", "mojang")
                        .replace("${version_type}", "release");
                    cmd.arg(substituted);
                }
            }
        }
    }

    log::info!("Launching Minecraft {}...", MC_VERSION);
    log::debug!("Command: {:?}", cmd);
    cmd.spawn()?;

    Ok(())
}

// ============================================================================
// Utility Functions
// ============================================================================

async fn download_file(client: &Client, url: &str, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let response = client.get(url).send().await?;
    let bytes = response.bytes().await?;
    tokio::fs::write(path, bytes).await?;

    Ok(())
}

// ============================================================================
// Public API
// ============================================================================

pub async fn launch(
    username: String,
    uuid: String,
    token: String,
) -> Result<()> {
    let base_dir = PathBuf::from("ezlauncher_data");

    // Step 1: Install Java
    let java_path = install_java(&base_dir).await?;
    log::info!("Java ready at: {:?}", java_path);

    // Step 2: Install Minecraft
    let (mc_dir, manifest) = install_minecraft(&base_dir).await?;
    log::info!("Minecraft installed");

    // Step 3: Launch game
    launch_game(mc_dir, java_path, manifest, username, uuid, token).await?;

    Ok(())
}
