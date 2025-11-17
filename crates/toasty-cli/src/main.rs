use clap::{Parser, Subcommand};
use anyhow::Result;
use toasty_migrate::*;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "toasty")]
#[command(about = "Toasty ORM CLI - Database migration and schema management", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new migration from schema changes
    #[command(name = "migrate:generate")]
    MigrateGenerate {
        /// Description of the migration
        #[arg(short, long)]
        message: String,

        /// Path to migrations directory
        #[arg(short, long, default_value = "migrations")]
        dir: String,
    },

    /// Run pending migrations
    #[command(name = "migrate:up")]
    MigrateUp {
        /// Database connection URL
        #[arg(short, long)]
        url: String,

        /// Path to migrations directory
        #[arg(short, long, default_value = "migrations")]
        dir: String,
    },

    /// Rollback migrations
    #[command(name = "migrate:down")]
    MigrateDown {
        /// Database connection URL
        #[arg(short, long)]
        url: String,

        /// Number of migrations to rollback
        #[arg(short, long, default_value = "1")]
        count: usize,

        /// Path to migrations directory
        #[arg(short, long, default_value = "migrations")]
        dir: String,
    },

    /// Show migration status
    #[command(name = "migrate:status")]
    MigrateStatus {
        /// Database connection URL
        #[arg(short, long)]
        url: String,

        /// Path to migrations directory
        #[arg(short, long, default_value = "migrations")]
        dir: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::MigrateGenerate { message, dir } => {
            cmd_generate(message, dir).await
        }
        Commands::MigrateUp { url, dir } => {
            cmd_up(url, dir).await
        }
        Commands::MigrateDown { url, count, dir } => {
            cmd_down(url, count, dir).await
        }
        Commands::MigrateStatus { url, dir } => {
            cmd_status(url, dir).await
        }
    }
}

async fn cmd_generate(message: String, dir: String) -> Result<()> {
    println!("🔍 Generating migration: {}", message);
    println!("📁 Migration directory: {}", dir);
    println!();

    let migration_dir = PathBuf::from(&dir);
    let loader = MigrationLoader::new(&migration_dir);
    let snapshot_path = loader.snapshot_path();

    // Load old snapshot (or create empty if first migration)
    let old_snapshot = if snapshot_path.exists() {
        println!("📸 Loading existing schema snapshot...");
        load_snapshot(&snapshot_path)?
    } else {
        println!("📸 No existing snapshot found, creating baseline...");
        SchemaSnapshot {
            version: "1.0".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            tables: vec![],
        }
    };

    // TODO: Build new snapshot from current models
    // For now, this requires schema from the project's models
    // In a real implementation, we'd need to either:
    // 1. Load the schema from the compiled project
    // 2. Parse model files directly
    // 3. Require user to provide schema programmatically

    println!("⚠️  Note: Schema building from models requires project integration");
    println!("   For now, you can use the toasty_migrate library programmatically");
    println!();
    println!("Example usage in build.rs or custom tool:");
    println!("```rust");
    println!("let schema = build_schema_from_models(); // Your schema building code");
    println!("let new_snapshot = SchemaSnapshot::from_schema(&schema);");
    println!("let diff = detect_changes(&old_snapshot, &new_snapshot)?;");
    println!("let generator = MigrationGenerator::new(\"{}\");", dir);
    println!("let migration = generator.generate(&diff, \"{}\")?;", message);
    println!("generator.write_migration_file(&migration)?;");
    println!("save_snapshot(&new_snapshot, snapshot_path)?;");
    println!("```");

    Ok(())
}

async fn cmd_up(_url: String, _dir: String) -> Result<()> {
    println!("⬆️  Running migrations...");
    println!();

    println!("⚠️  Note: Migration execution requires database connection");
    println!("   The migration runner is fully implemented in toasty-migrate");
    println!();
    println!("Example usage:");
    println!("```rust");
    println!("let mut tracker = MigrationTracker::new();");
    println!("let mut runner = MigrationRunner::new(tracker);");
    println!("runner.initialize().await?;");
    println!();
    println!("let loader = MigrationLoader::new(\"migrations\");");
    println!("let migration_files = loader.discover_migrations()?;");
    println!("let migrations: Vec<Box<dyn Migration>> = load_migrations(migration_files);");
    println!();
    println!("let mut context = SqlMigrationContext::new(SqlFlavor::Sqlite);");
    println!("runner.run_pending(migrations, &mut context).await?;");
    println!("```");

    Ok(())
}

async fn cmd_down(_url: String, _count: usize, _dir: String) -> Result<()> {
    println!("⬇️  Rolling back migrations...");
    println!();

    println!("⚠️  Note: Migration rollback requires database connection");
    println!("   The rollback logic is fully implemented in toasty-migrate");
    println!();
    println!("Example usage:");
    println!("```rust");
    println!("let mut runner = MigrationRunner::new(tracker);");
    println!("let mut context = SqlMigrationContext::new(SqlFlavor::Sqlite);");
    println!("runner.rollback(count, migrations, &mut context).await?;");
    println!("```");

    Ok(())
}

async fn cmd_status(_url: String, dir: String) -> Result<()> {
    println!("📊 Migration Status");
    println!("📁 Migration directory: {}", dir);
    println!();

    let loader = MigrationLoader::new(PathBuf::from(&dir));
    let migration_files = loader.discover_migrations()?;

    if migration_files.is_empty() {
        println!("No migrations found in {}", dir);
        return Ok(());
    }

    println!("Found {} migration file(s):\n", migration_files.len());
    println!("Version                      | Filename");
    println!("---------------------------- | --------");

    for file in &migration_files {
        println!("{:28} | {}", file.version, file.filename);
    }

    println!();
    println!("⚠️  Note: Applied/pending status requires database connection");
    println!("   Migration tracking is fully implemented in toasty-migrate");

    Ok(())
}
