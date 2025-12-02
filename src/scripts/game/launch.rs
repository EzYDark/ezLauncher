use super::types::*;
use super::utils::check_rules;
use super::{ELY_BY_API, MC_VERSION};
use anyhow::Result;
use std::path::PathBuf;

pub async fn launch_game(
    mc_dir: PathBuf,
    java_path: PathBuf,
    manifest: VersionManifest,
    username: String,
    uuid: String,
    token: String,
) -> Result<()> {
    log::info!("Building launch command...");

    // Parse ignoreList from JVM args to exclude libraries from classpath
    let mut ignore_list: Vec<String> = Vec::new();
    if let Some(args) = &manifest.arguments {
        if let Some(jvm_args) = &args.jvm {
            for arg in jvm_args {
                if let serde_json::Value::String(s) = arg {
                    if s.starts_with("-DignoreList=") {
                        let list = s.trim_start_matches("-DignoreList=");
                        let list = list.replace("${version_name}", MC_VERSION); // Substitute version name
                        ignore_list = list.split(',').map(|s| s.to_string()).collect();
                        log::info!("Ignore list: {:?}", ignore_list);
                        break;
                    }
                }
            }
        }
    }

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
                let lib_filename = lib_path.file_name().unwrap_or_default().to_string_lossy();

                // Skip if in ignore list
                if ignore_list.iter().any(|ignore| {
                    lib_filename == *ignore
                        || (ignore.ends_with("-") && lib_filename.starts_with(ignore))
                }) {
                    log::info!("Ignoring library: {}", lib_filename);
                    continue;
                }

                if lib_path.exists() {
                    if let Ok(absolute) = std::fs::canonicalize(&lib_path) {
                        classpath.push(
                            absolute
                                .to_string_lossy()
                                .trim_start_matches(r"\\?\")
                                .to_string(),
                        );
                    }
                }
            }
        }
    }

    // Add client JAR (absolute path)
    // Only add if not in ignore list (NeoForge puts client jar on module path usually)
    let client_jar_name = format!("{}.jar", MC_VERSION);
    if !ignore_list.contains(&client_jar_name) {
        let client_jar = mc_dir
            .join("versions")
            .join(MC_VERSION)
            .join(&client_jar_name);
        if let Ok(absolute) = std::fs::canonicalize(&client_jar) {
            classpath.push(
                absolute
                    .to_string_lossy()
                    .trim_start_matches(r"\\?\")
                    .to_string(),
            );
        }
    } else {
        log::info!("Ignoring client JAR: {}", client_jar_name);
    }

    // Deduplicate classpath entries
    classpath.sort();
    classpath.dedup();

    let separator = if cfg!(target_os = "windows") {
        ";"
    } else {
        ":"
    };
    let classpath_str = classpath.join(separator);
    log::info!("Classpath: {}", classpath_str);

    // Build command
    let java_absolute = std::fs::canonicalize(&java_path)?;
    let mc_absolute = std::fs::canonicalize(&mc_dir)?;
    let natives_absolute = std::fs::canonicalize(mc_dir.join("natives"))?;

    let mut cmd = tokio::process::Command::new(&java_absolute);
    cmd.current_dir(&mc_absolute);

    // Memory args
    cmd.arg("-Xmx4G").arg("-Xms1G");

    // Authlib-injector for Ely.by authentication
    let authlib_path = mc_dir
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Invalid mc_dir"))?
        .join("authlib-injector.jar");
    if let Ok(authlib_absolute) = std::fs::canonicalize(&authlib_path) {
        cmd.arg(format!(
            "-javaagent:{}={}",
            authlib_absolute
                .to_string_lossy()
                .trim_start_matches(r"\\?\"),
            ELY_BY_API
        ));
    }

    // Native library path
    cmd.arg(format!(
        "-Djava.library.path={}",
        natives_absolute
            .to_string_lossy()
            .trim_start_matches(r"\\?\")
    ));

    // Classpath
    cmd.arg("-cp").arg(&classpath_str);

    // JVM arguments from manifest (with variable substitution)
    if let Some(args) = &manifest.arguments {
        if let Some(jvm_args) = &args.jvm {
            for arg in jvm_args {
                if let serde_json::Value::String(s) = arg {
                    let substituted = s
                        .replace(
                            "${natives_directory}",
                            natives_absolute
                                .to_string_lossy()
                                .trim_start_matches(r"\\?\"),
                        )
                        .replace("${launcher_name}", "ezLauncher")
                        .replace("${launcher_version}", "0.2.0")
                        .replace("${classpath}", &classpath_str)
                        .replace(
                            "${library_directory}",
                            mc_absolute
                                .join("libraries")
                                .to_string_lossy()
                                .trim_start_matches(r"\\?\"),
                        )
                        .replace("${classpath_separator}", separator)
                        .replace("${version_name}", MC_VERSION);
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
                        .replace(
                            "${game_directory}",
                            mc_absolute.to_string_lossy().trim_start_matches(r"\\?\"),
                        )
                        .replace(
                            "${assets_root}",
                            mc_absolute
                                .join("assets")
                                .to_string_lossy()
                                .trim_start_matches(r"\\?\"),
                        )
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
