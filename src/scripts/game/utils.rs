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

pub fn extract_natives(jar_path: &Path, natives_dir: &Path) -> Result<()> {
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
