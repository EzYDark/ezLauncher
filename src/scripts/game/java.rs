use super::utils::download_file;
use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use std::path::{Path, PathBuf};

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

pub async fn install_java(base_dir: &Path) -> Result<PathBuf> {
    let java_dir = base_dir.join("java");
    let jdk_dir = java_dir.join("jdk-21.0.9+10");

    let java_bin = if cfg!(target_os = "windows") {
        jdk_dir.join("bin").join("javaw.exe")
    } else {
        jdk_dir.join("bin").join("java")
    };

    if java_bin.exists() {
        log::info!("Java already installed");
        return Ok(java_bin);
    }

    log::info!("Downloading Java 21...");
    let client = Client::builder().user_agent("ezLauncher/0.2.0").build()?;

    let (url, ext) = if cfg!(target_os = "windows") {
        (
            "https://api.adoptium.net/v3/assets/latest/21/hotspot?architecture=x64&image_type=jdk&os=windows&vendor=eclipse",
            "zip",
        )
    } else {
        (
            "https://api.adoptium.net/v3/assets/latest/21/hotspot?architecture=x64&image_type=jdk&os=linux&vendor=eclipse",
            "tar.gz",
        )
    };

    let releases: Vec<AdoptiumRelease> = client.get(url).send().await?.json().await?;

    let release = releases
        .first()
        .ok_or_else(|| anyhow::anyhow!("No Java releases found"))?;

    let archive_path = java_dir.join(format!("jdk.{}", ext));
    download_file(&client, &release.binary.package.link, &archive_path).await?;

    log::info!("Extracting Java...");
    tokio::fs::create_dir_all(&java_dir).await?;

    if cfg!(target_os = "windows") {
        let archive_path_clone = archive_path.clone();
        let java_dir_clone = java_dir.clone();
        tokio::task::spawn_blocking(move || {
            super::utils::extract_zip(&archive_path_clone, &java_dir_clone)
        })
        .await??;
    } else {
        tokio::process::Command::new("tar")
            .arg("-xzf")
            .arg(&archive_path)
            .arg("-C")
            .arg(&java_dir)
            .status()
            .await?;
    }

    tokio::fs::remove_file(archive_path).await?;
    Ok(java_bin)
}
