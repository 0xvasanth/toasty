use anyhow::Result;
use std::path::Path;

/// Loads migration files from a directory
pub struct MigrationLoader {
    migration_dir: std::path::PathBuf,
}

impl MigrationLoader {
    pub fn new(migration_dir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            migration_dir: migration_dir.into(),
        }
    }

    /// Discover all migration files in the directory
    pub fn discover_migrations(&self) -> Result<Vec<MigrationFileInfo>> {
        let mut migrations = Vec::new();

        if !self.migration_dir.exists() {
            return Ok(migrations);
        }

        for entry in std::fs::read_dir(&self.migration_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    // Extract version from filename
                    // Format: YYYYMMDD_HHMMSS_description.rs
                    if let Some(version) = filename.strip_suffix(".rs") {
                        migrations.push(MigrationFileInfo {
                            version: version.to_string(),
                            path: path.clone(),
                            filename: filename.to_string(),
                        });
                    }
                }
            }
        }

        // Sort by version (timestamp-based)
        migrations.sort_by(|a, b| a.version.cmp(&b.version));

        Ok(migrations)
    }

    /// Get path to schema snapshot file
    pub fn snapshot_path(&self) -> std::path::PathBuf {
        self.migration_dir.join(".schema.json")
    }
}

#[derive(Debug, Clone)]
pub struct MigrationFileInfo {
    pub version: String,
    pub path: std::path::PathBuf,
    pub filename: String,
}
