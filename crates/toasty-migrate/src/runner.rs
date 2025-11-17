use crate::{Migration, MigrationContext, MigrationTracker};
use anyhow::Result;

/// Executes migrations against a database
pub struct MigrationRunner {
    tracker: MigrationTracker,
}

impl MigrationRunner {
    pub fn new(tracker: MigrationTracker) -> Self {
        Self { tracker }
    }

    /// Initialize the migration system (create tracking table)
    pub async fn initialize(&mut self) -> Result<()> {
        self.tracker.initialize().await?;
        self.tracker.load_applied().await?;
        Ok(())
    }

    /// Run all pending migrations
    pub async fn run_pending(
        &mut self,
        migrations: Vec<Box<dyn Migration>>,
        context: &mut dyn MigrationContext,
    ) -> Result<usize> {
        let mut applied_count = 0;

        for migration in migrations {
            let version = migration.version();

            // Skip if already applied
            if self.tracker.is_applied(version) {
                continue;
            }

            println!("Applying migration: {}", version);

            // Execute the up migration
            migration.up(context)?;

            // Mark as applied
            self.tracker.mark_applied(version.to_string());
            self.tracker.persist_applied(version).await?;

            applied_count += 1;
            println!("  ✅ Applied: {}", version);
        }

        if applied_count == 0 {
            println!("No pending migrations");
        } else {
            println!("\n✅ Applied {} migration(s)", applied_count);
        }

        Ok(applied_count)
    }

    /// Rollback the last N migrations
    pub async fn rollback(
        &mut self,
        count: usize,
        migrations: Vec<Box<dyn Migration>>,
        context: &mut dyn MigrationContext,
    ) -> Result<usize> {
        let applied = self.tracker.applied_migrations();

        if applied.is_empty() {
            println!("No migrations to rollback");
            return Ok(0);
        }

        let to_rollback = applied.iter().rev().take(count);
        let mut rolled_back_count = 0;

        for version in to_rollback {
            // Find the migration
            let migration = migrations
                .iter()
                .find(|m| m.version() == version)
                .ok_or_else(|| anyhow::anyhow!("Migration not found: {}", version))?;

            println!("Rolling back migration: {}", version);

            // Execute the down migration
            migration.down(context)?;

            // Mark as rolled back
            self.tracker.mark_rolled_back(version);
            self.tracker.persist_rolled_back(version).await?;

            rolled_back_count += 1;
            println!("  ✅ Rolled back: {}", version);
        }

        println!("\n✅ Rolled back {} migration(s)", rolled_back_count);
        Ok(rolled_back_count)
    }

    /// Get migration status
    pub fn status(&self, all_migrations: Vec<Box<dyn Migration>>) -> Vec<MigrationStatus> {
        all_migrations
            .into_iter()
            .map(|migration| {
                let version = migration.version().to_string();
                let applied = self.tracker.is_applied(&version);

                MigrationStatus {
                    version,
                    applied,
                    applied_at: None, // TODO: Get from database
                }
            })
            .collect()
    }

    pub fn tracker(&self) -> &MigrationTracker {
        &self.tracker
    }
}

#[derive(Debug)]
pub struct MigrationStatus {
    pub version: String,
    pub applied: bool,
    pub applied_at: Option<String>,
}
