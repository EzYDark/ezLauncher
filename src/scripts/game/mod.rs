pub mod types;
pub mod utils;
pub mod java;
pub mod install;
pub mod launch;

use anyhow::Result;
use std::path::PathBuf;
pub use types::VersionType;
use java::install_java;
use install::install_minecraft;
use launch::launch_game;

pub const VERSION_MANIFEST_URL: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
pub const MC_VERSION: &str = "1.21.1";
pub const NEOFORGE_VERSION: &str = "21.1.65";
pub const AUTHLIB_INJECTOR_URL: &str = "https://github.com/yushijinhun/authlib-injector/releases/download/v1.2.5/authlib-injector-1.2.5.jar";
pub const ELY_BY_API: &str = "https://authserver.ely.by/api/authlib-injector";

pub async fn launch(
    username: String,
    uuid: String,
    token: String,
    version_type: VersionType,
) -> Result<()> {
    let base_dir = PathBuf::from("ezlauncher_data");

    // Step 1: Install Java
    let java_path = install_java(&base_dir).await?;
    log::info!("Java ready at: {:?}", java_path);

    // Step 2: Install Minecraft
    let (mc_dir, manifest) = install_minecraft(&base_dir, &java_path, version_type).await?;
    log::info!("Minecraft installed");

    // Step 3: Launch game
    launch_game(mc_dir, java_path, manifest, username, uuid, token).await?;

    Ok(())
}
