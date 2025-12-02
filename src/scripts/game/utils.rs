use anyhow::Result;
use reqwest::Client;
use std::path::Path;
use super::types::Rule;

pub async fn download_file(client: &Client, url: &str, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let response = client.get(url).send().await?;
    let bytes = response.bytes().await?;
    tokio::fs::write(path, bytes).await?;

    Ok(())
}

pub fn check_rules(rules: &[Rule]) -> bool {
    let current_os = std::env::consts::OS;
    let mut allowed = false;
    
    for rule in rules {
        if rule.action == "allow" {
            if let Some(os) = &rule.os {
                if let Some(name) = &os.name {
                    if name == current_os || (name == "osx" && current_os == "macos") {
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
                    if name == current_os || (name == "osx" && current_os == "macos") {
                        allowed = false;
                    }
                }
            }
        }
    }
    allowed
}

pub fn extract_natives(jar_path: &Path, natives_dir: &Path) -> Result<()> {
    let file = std::fs::File::open(jar_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_owned();

        // Skip META-INF and directories
        if name.starts_with("META-INF") || file.is_dir() {
            continue;
        }

        // Filter by extension based on OS
        let is_valid_native = if cfg!(target_os = "windows") {
            name.ends_with(".dll")
        } else if cfg!(target_os = "linux") {
            name.ends_with(".so")
        } else if cfg!(target_os = "macos") {
            name.ends_with(".dylib")
        } else {
            false
        };

        if !is_valid_native {
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

pub fn extract_zip(zip_path: &Path, out_dir: &Path) -> Result<()> {
    let file = std::fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_owned();

        // Skip invalid paths
        if name.contains("..") {
            continue;
        }

        let out_path = out_dir.join(&name);
        
        if file.is_dir() {
            std::fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = std::fs::File::create(&out_path)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
        
        // Preserve permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                std::fs::set_permissions(&out_path, std::fs::Permissions::from_mode(mode))?;
            }
        }
    }

    Ok(())
}
