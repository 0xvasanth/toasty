use anyhow::Result;
use std::collections::HashSet;

/// Tracks which migrations have been applied to the database
pub struct MigrationTracker {
    applied: HashSet<String>,
}

impl MigrationTracker {
    pub fn new() -> Self {
        Self {
            applied: HashSet::new(),
        }
    }

    /// Initialize migration tracking table
    /// SQL: CREATE TABLE IF NOT EXISTS _toasty_migrations (
    ///         version VARCHAR(255) PRIMARY KEY,
    ///         applied_at TIMESTAMP NOT NULL
    ///      )
    /// NoSQL: Create collection with version as primary key
    pub async fn initialize(&mut self) -> Result<()> {
        // TODO: Execute database-specific table/collection creation
        // This will be implemented in the MigrationContext
        Ok(())
    }

    /// Load applied migrations from database
    pub async fn load_applied(&mut self) -> Result<()> {
        // TODO: Query _toasty_migrations table/collection
        // For now, returns empty set
        Ok(())
    }

    /// Check if a migration has been applied
    pub fn is_applied(&self, version: &str) -> bool {
        self.applied.contains(version)
    }

    /// Mark a migration as applied
    pub fn mark_applied(&mut self, version: String) {
        self.applied.insert(version);
    }

    /// Mark a migration as rolled back
    pub fn mark_rolled_back(&mut self, version: &str) {
        self.applied.remove(version);
    }

    /// Get all applied migrations
    pub fn applied_migrations(&self) -> Vec<String> {
        let mut migrations: Vec<_> = self.applied.iter().cloned().collect();
        migrations.sort();
        migrations
    }

    /// Persist applied migration to database
    pub async fn persist_applied(&self, _version: &str) -> Result<()> {
        // TODO: INSERT INTO _toasty_migrations (version, applied_at)
        Ok(())
    }

    /// Remove migration record from database
    pub async fn persist_rolled_back(&self, _version: &str) -> Result<()> {
        // TODO: DELETE FROM _toasty_migrations WHERE version = ?
        Ok(())
    }
}
