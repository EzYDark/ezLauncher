use anyhow::{Result};

pub struct AppDir {
    pub name: &'static str,
    pub path: std::path::PathBuf,
}

impl AppDir {
    pub fn new(name: &'static str) -> Self {
        let mut path = dirs::home_dir().unwrap();
        path.push(name);
        Self { name, path }
    }

    pub fn ensure_exists(&self) -> Result<()> {
        if !self.path.exists() {
             std::fs::create_dir_all(&self.path)?;
        }
        Ok(())
    }

    pub fn get_path(&self) -> &std::path::PathBuf {
        &self.path
    }

    pub fn get_name(&self) -> &str {
        self.name
    }
}