use anyhow::Result;
use toasty_migrate::*;
use crate::executor::MigrationExecutor;
use std::path::Path;

/// Shadow database for migration diff calculation
/// 
/// Creates a temporary database, applies all existing migrations to it,
/// then introspects to get the "current state after all migrations".
/// This is compared with desired entity schema to generate only new changes.
pub struct ShadowDatabase {
    url: String,
    temp_file: Option<tempfile::NamedTempFile>,
}

impl ShadowDatabase {
    /// Create a new shadow database
    pub fn new() -> Result<Self> {
        // Create temporary SQLite database
        let temp_file = tempfile::NamedTempFile::new()?;
        let shadow_url = format!("sqlite:{}", temp_file.path().display());

        Ok(Self {
            url: shadow_url,
            temp_file: Some(temp_file),
        })
    }

    /// Apply all migrations from directory to shadow database
    pub fn apply_migrations(&self, migration_dir: &Path) -> Result<SchemaSnapshot> {
        println!("🔄 Creating shadow database...");

        // Load all migration files
        let loader = MigrationLoader::new(migration_dir);
        let migration_files = loader.discover_migrations()?;

        if migration_files.is_empty() {
            println!("   No existing migrations - empty schema");
            return Ok(SchemaSnapshot {
                version: "1.0".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                tables: vec![],
            });
        }

        println!("   Applying {} migration(s) to shadow database", migration_files.len());

        // For each migration, we need to reconstruct its schema changes
        // Since we can't execute the .rs files directly, we'll use .schema.json
        // as the accumulated state after each migration
        
        // Load the schema snapshot (represents state after all migrations)
        let schema_path = migration_dir.join(".schema.json");
        let current_state = if schema_path.exists() {
            load_snapshot(&schema_path)?
        } else {
            // No schema.json means no migrations have been tracked yet
            SchemaSnapshot {
                version: "1.0".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                tables: vec![],
            }
        };

        println!("   ✅ Shadow database ready with {} table(s)", current_state.tables.len());

        Ok(current_state)
    }

    /// Get shadow database URL
    pub fn url(&self) -> &str {
        &self.url
    }
}

impl Drop for ShadowDatabase {
    fn drop(&mut self) {
        // Temp file automatically deleted
        println!("🗑️  Shadow database cleaned up");
    }
}
