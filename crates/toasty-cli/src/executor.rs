use anyhow::Result;
use toasty_migrate::*;

/// Execute SQL migrations against a database
pub struct MigrationExecutor {
    url: String,
}

impl MigrationExecutor {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    /// Execute a migration context's statements against the database
    #[cfg(feature = "postgresql")]
    pub async fn execute_postgresql(&self, context: &SqlMigrationContext) -> Result<()> {
        use tokio_postgres::NoTls;

        println!("🔌 Connecting to PostgreSQL...");
        let (client, connection) = tokio_postgres::connect(&self.url, NoTls).await?;

        // Spawn connection
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });

        // Execute each SQL statement
        for (i, sql) in context.statements().iter().enumerate() {
            println!("   Executing statement {}: {}", i + 1, sql.lines().next().unwrap_or(sql));
            client.execute(sql, &[]).await?;
        }

        println!("✅ Executed {} statement(s)", context.statements().len());
        Ok(())
    }

    #[cfg(not(feature = "postgresql"))]
    pub async fn execute_postgresql(&self, _context: &SqlMigrationContext) -> Result<()> {
        Err(anyhow::anyhow!("PostgreSQL support not enabled"))
    }

    /// Drop all tables in the database
    #[cfg(feature = "postgresql")]
    pub async fn drop_all_tables_postgresql(&self) -> Result<usize> {
        use tokio_postgres::NoTls;

        let (client, connection) = tokio_postgres::connect(&self.url, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });

        // Get all tables
        let rows = client.query(
            "SELECT tablename FROM pg_tables WHERE schemaname = 'public'",
            &[],
        ).await?;

        let mut dropped = 0;
        for row in rows {
            let table_name: String = row.get(0);

            // Skip migration tracking table
            if table_name == "_toasty_migrations" {
                continue;
            }

            println!("   Dropping table: {}", table_name);
            client.execute(&format!("DROP TABLE IF EXISTS {} CASCADE", table_name), &[]).await?;
            dropped += 1;
        }

        Ok(dropped)
    }

    #[cfg(not(feature = "postgresql"))]
    pub async fn drop_all_tables_postgresql(&self) -> Result<usize> {
        Err(anyhow::anyhow!("PostgreSQL support not enabled"))
    }

    /// Create migration tracking table
    #[cfg(feature = "postgresql")]
    pub async fn create_tracking_table_postgresql(&self) -> Result<()> {
        use tokio_postgres::NoTls;

        let (client, connection) = tokio_postgres::connect(&self.url, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });

        client.execute(
            "CREATE TABLE IF NOT EXISTS _toasty_migrations (
                version VARCHAR(255) PRIMARY KEY,
                applied_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
            &[],
        ).await?;

        Ok(())
    }

    /// Check if migration is applied
    #[cfg(feature = "postgresql")]
    pub async fn is_migration_applied_postgresql(&self, version: &str) -> Result<bool> {
        use tokio_postgres::NoTls;

        let (client, connection) = tokio_postgres::connect(&self.url, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });

        let rows = client.query(
            "SELECT 1 FROM _toasty_migrations WHERE version = $1",
            &[&version],
        ).await?;

        Ok(!rows.is_empty())
    }

    /// Mark migration as applied
    #[cfg(feature = "postgresql")]
    pub async fn mark_migration_applied_postgresql(&self, version: &str) -> Result<()> {
        use tokio_postgres::NoTls;

        let (client, connection) = tokio_postgres::connect(&self.url, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });

        client.execute(
            "INSERT INTO _toasty_migrations (version) VALUES ($1)",
            &[&version],
        ).await?;

        Ok(())
    }

    /// Remove migration record
    #[cfg(feature = "postgresql")]
    pub async fn mark_migration_rolled_back_postgresql(&self, version: &str) -> Result<()> {
        use tokio_postgres::NoTls;

        let (client, connection) = tokio_postgres::connect(&self.url, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });

        client.execute(
            "DELETE FROM _toasty_migrations WHERE version = $1",
            &[&version],
        ).await?;

        Ok(())
    }
}
