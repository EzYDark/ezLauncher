use super::types::*;
use super::utils::{check_rules, download_file, extract_natives};
use super::{AUTHLIB_INJECTOR_URL, MC_VERSION, NEOFORGE_VERSION, VERSION_MANIFEST_URL};
use anyhow::Result;
use reqwest::Client;
use std::path::{Path, PathBuf};

pub async fn install_minecraft(
    base_dir: &Path,
    java_path: &Path,
    version_type: VersionType,
) -> Result<(PathBuf, VersionManifest)> {
    let mc_dir = base_dir.join("minecraft");

    // Always install vanilla base first
    install_vanilla_base(base_dir).await?;

    let manifest_path = match version_type {
        VersionType::Vanilla => mc_dir
            .join("versions")
            .join(MC_VERSION)
            .join(format!("{}.json", MC_VERSION)),
        VersionType::NeoForge => {
            install_neoforge(base_dir, java_path).await?;
            mc_dir
                .join("versions")
                .join(format!("neoforge-{}", NEOFORGE_VERSION))
                .join(format!("neoforge-{}.json", NEOFORGE_VERSION))
        }
    };

    let manifest_content = tokio::fs::read_to_string(&manifest_path).await?;
    let mut manifest: VersionManifest = serde_json::from_str(&manifest_content)?;

    // Handle inheritance (e.g. NeoForge inherits from Vanilla)
    if let Some(parent_id) = &manifest.inherits_from {
        let parent_path = mc_dir
            .join("versions")
            .join(parent_id)
            .join(format!("{}.json", parent_id));

        if parent_path.exists() {
            log::info!("Inheriting libraries from {}", parent_id);
            let parent_content = tokio::fs::read_to_string(&parent_path).await?;
            let parent_manifest: VersionManifest = serde_json::from_str(&parent_content)?;

            // Create a set of existing library names to prevent duplicates
            let existing_libs: std::collections::HashSet<String> =
                manifest.libraries.iter().map(|l| l.name.clone()).collect();

            for lib in parent_manifest.libraries {
                if !existing_libs.contains(&lib.name) {
                    manifest.libraries.push(lib);
                }
            }

            // Merge game arguments
            if let Some(parent_args) = parent_manifest.arguments {
                if let Some(parent_game_args) = parent_args.game {
                    if let Some(args) = &mut manifest.arguments {
                        if let Some(game_args) = &mut args.game {
                            game_args.extend(parent_game_args);
                        } else {
                            args.game = Some(parent_game_args);
                        }
                    } else {
                        manifest.arguments = Some(Arguments {
                            game: Some(parent_game_args),
                            jvm: None, // Don't inherit JVM args, NeoForge provides its own
                        });
                    }
                }
            }
            // Inherit asset index
            if manifest.asset_index.is_none() {
                manifest.asset_index = parent_manifest.asset_index;
            }
        }
    }

    // Download libraries and extract natives for the selected version
    download_libraries_and_natives(&mc_dir, &manifest).await?;

    // Download assets
    download_assets(&mc_dir, &manifest).await?;

    Ok((mc_dir, manifest))
}

async fn install_vanilla_base(base_dir: &Path) -> Result<()> {
    let mc_dir = base_dir.join("minecraft");
    let version_dir = mc_dir.join("versions").join(MC_VERSION);
    let version_json_path = version_dir.join(format!("{}.json", MC_VERSION));
    let client_jar_path = version_dir.join(format!("{}.jar", MC_VERSION));

    let client = Client::builder().user_agent("ezLauncher/0.2.0").build()?;

    // Step 1: Get version manifest index
    if !version_json_path.exists() {
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

        // Save version JSON
        tokio::fs::create_dir_all(&version_dir).await?;
        tokio::fs::write(&version_json_path, &version_json).await?;
    }

    // Step 4: Download client JAR
    if !client_jar_path.exists() {
        let manifest_content = tokio::fs::read_to_string(&version_json_path).await?;
        let manifest: VersionManifest = serde_json::from_str(&manifest_content)?;

        if let Some(downloads) = &manifest.downloads {
            log::info!("Downloading client JAR...");
            download_file(&client, &downloads.client.url, &client_jar_path).await?;
        }
    }

    // Step 7: Download authlib-injector for Ely.by authentication
    let authlib_path = base_dir.join("authlib-injector.jar");
    if !authlib_path.exists() {
        log::info!("Downloading authlib-injector...");
        download_file(&client, AUTHLIB_INJECTOR_URL, &authlib_path).await?;
    }

    Ok(())
}

async fn download_libraries_and_natives(mc_dir: &Path, manifest: &VersionManifest) -> Result<()> {
    let client = Client::builder().user_agent("ezLauncher/0.2.0").build()?;
    let lib_dir = mc_dir.join("libraries");

    // Download libraries
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
                let os_key = if cfg!(target_os = "windows") {
                    "windows"
                } else {
                    "linux"
                };
                if let Some(classifier) = natives.get(os_key) {
                    if let Some(classifiers) = &downloads.classifiers {
                        if let Some(native_artifact) = classifiers.get(classifier) {
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

    // Extract natives
    let natives_dir = mc_dir.join("natives");
    tokio::fs::create_dir_all(&natives_dir).await?;

    for library in &manifest.libraries {
        if let Some(natives) = &library.natives {
            let os_key = if cfg!(target_os = "windows") {
                "windows"
            } else {
                "linux"
            };
            if let Some(classifier) = natives.get(os_key) {
                if let Some(downloads) = &library.downloads {
                    if let Some(classifiers) = &downloads.classifiers {
                        if let Some(native_artifact) = classifiers.get(classifier) {
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

    Ok(())
}

async fn download_assets(mc_dir: &Path, manifest: &VersionManifest) -> Result<()> {
    if let Some(asset_index) = &manifest.asset_index {
        let assets_dir = mc_dir.join("assets");
        let indexes_dir = assets_dir.join("indexes");
        let objects_dir = assets_dir.join("objects");

        tokio::fs::create_dir_all(&indexes_dir).await?;
        tokio::fs::create_dir_all(&objects_dir).await?;

        let index_path = indexes_dir.join(format!("{}.json", asset_index.id));
        let client = Client::builder().user_agent("ezLauncher/0.2.0").build()?;

        if !index_path.exists() {
            log::info!("Downloading asset index: {}", asset_index.id);
            download_file(&client, &asset_index.url, &index_path).await?;
        }

        let index_content = tokio::fs::read_to_string(&index_path).await?;
        let index: AssetsIndex = serde_json::from_str(&index_content)?;

        log::info!("Downloading assets...");
        // Download assets in parallel
        let mut tasks = Vec::new();
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(20)); // Limit concurrent downloads

        for (name, object) in index.objects {
            let hash_head = object.hash[0..2].to_string();
            let object_path = objects_dir.join(&hash_head).join(&object.hash);
            let client = client.clone();
            let semaphore = semaphore.clone();

            if !object_path.exists() {
                tasks.push(tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    let url = format!(
                        "https://resources.download.minecraft.net/{}/{}",
                        hash_head, object.hash
                    );
                    // log::debug!("Downloading asset: {}", name); // Too verbose
                    if let Err(e) = download_file(&client, &url, &object_path).await {
                        log::error!("Failed to download asset {}: {}", name, e);
                    }
                }));
            }
        }

        for task in tasks {
            task.await?;
        }
    }
    Ok(())
}

async fn install_neoforge(base_dir: &Path, java_path: &Path) -> Result<()> {
    let installer_url = format!(
        "https://maven.neoforged.net/releases/net/neoforged/neoforge/{}/neoforge-{}-installer.jar",
        NEOFORGE_VERSION, NEOFORGE_VERSION
    );
    let installer_path = base_dir.join(format!("neoforge-{}-installer.jar", NEOFORGE_VERSION));

    if !installer_path.exists() {
        log::info!("Downloading NeoForge installer...");
        let client = Client::builder().user_agent("ezLauncher/0.2.0").build()?;
        download_file(&client, &installer_url, &installer_path).await?;
    }

    let mc_dir = base_dir.join("minecraft");

    // Create dummy launcher_profiles.json because NeoForge installer requires it
    let profiles_path = mc_dir.join("launcher_profiles.json");
    if !profiles_path.exists() {
        tokio::fs::write(&profiles_path, r#"{"profiles":{}}"#).await?;
    }

    let neoforge_version_dir = mc_dir
        .join("versions")
        .join(format!("neoforge-{}", NEOFORGE_VERSION));

    if !neoforge_version_dir.exists() {
        log::info!("Installing NeoForge...");
        let java_absolute = std::fs::canonicalize(java_path)?;
        let installer_absolute = std::fs::canonicalize(&installer_path)?;
        let mc_absolute = std::fs::canonicalize(&mc_dir)?;

        let status = tokio::process::Command::new(
            java_absolute.to_string_lossy().trim_start_matches(r"\\?\"),
        )
        .arg("-jar")
        .arg(
            installer_absolute
                .to_string_lossy()
                .trim_start_matches(r"\\?\"),
        )
        .arg("--installClient")
        .arg(mc_absolute.to_string_lossy().trim_start_matches(r"\\?\"))
        .status()
        .await?;

        if !status.success() {
            return Err(anyhow::anyhow!("NeoForge installer failed"));
        }
    }

    Ok(())
}
