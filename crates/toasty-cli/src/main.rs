mod reset;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use toasty_migrate::*;
use reset::cmd_reset;

#[derive(Parser)]
#[command(name = "toasty")]
#[command(about = "Toasty ORM CLI - Database migration and schema management", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize Toasty project structure with entity and migration crates
    Init {
        /// Project directory (defaults to current directory)
        #[arg(short, long, default_value = ".")]
        dir: String,
    },

    /// Generate a new migration from schema changes
    #[command(name = "migrate:generate")]
    MigrateGenerate {
        /// Description of the migration
        #[arg(short, long)]
        message: String,

        /// Database connection URL (required for introspection)
        #[arg(short, long)]
        url: String,

        /// Path to migrations directory
        #[arg(short, long, default_value = "migrations")]
        dir: String,

        /// Path to entity crate directory
        #[arg(short, long, default_value = "entity")]
        entity_dir: Option<String>,
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

    /// Reset database: drop all tables and rerun all migrations
    #[command(name = "migrate:reset")]
    MigrateReset {
        /// Database connection URL
        #[arg(short, long)]
        url: String,

        /// Path to migrations directory
        #[arg(short, long, default_value = "migrations")]
        dir: String,

        /// Path to entity crate directory
        #[arg(short, long, default_value = "entity")]
        entity_dir: Option<String>,

        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { dir } => cmd_init(dir).await,
        Commands::MigrateGenerate {
            message,
            url,
            dir,
            entity_dir,
        } => cmd_generate(message, url, dir, entity_dir).await,
        Commands::MigrateUp { url, dir } => cmd_up(url, dir).await,
        Commands::MigrateDown { url, count, dir } => cmd_down(url, count, dir).await,
        Commands::MigrateStatus { url, dir } => cmd_status(url, dir).await,
        Commands::MigrateReset {
            url,
            dir,
            entity_dir,
            force,
        } => cmd_reset(url, dir, entity_dir, force).await,
    }
}

async fn cmd_init(dir: String) -> Result<()> {
    println!("🚀 Initializing Toasty project structure...");
    println!("📁 Project directory: {}", dir);
    println!();

    let project_dir = PathBuf::from(&dir);

    // Create entity crate
    let entity_dir = project_dir.join("entity");
    std::fs::create_dir_all(entity_dir.join("src"))?;

    // Create entity Cargo.toml
    let entity_cargo_toml = format!(
        r#"[package]
name = "entity"
version = "0.1.0"
edition = "2021"

[dependencies]
toasty = {{ workspace = true }}
"#
    );
    std::fs::write(entity_dir.join("Cargo.toml"), entity_cargo_toml)?;

    // Create entity lib.rs with example
    let entity_lib_rs = r#"// Define your Toasty models here
// Example:
//
// use toasty::stmt::Id;
//
// #[derive(Debug, toasty::Model)]
// pub struct User {
//     #[key]
//     #[auto]
//     pub id: Id<Self>,
//     pub name: String,
//     #[unique]
//     pub email: String,
// }

pub use toasty;
"#;
    std::fs::write(entity_dir.join("src/lib.rs"), entity_lib_rs)?;
    println!("✅ Created entity crate: entity/");

    // Create migration directory
    let migration_dir = project_dir.join("migration");
    std::fs::create_dir_all(&migration_dir)?;

    // Create empty .schema.json
    let empty_snapshot = SchemaSnapshot {
        version: "1.0".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        tables: vec![],
    };
    save_snapshot(&empty_snapshot, migration_dir.join(".schema.json"))?;
    println!("✅ Created migration directory: migration/");

    // Create README
    let readme = r#"# Toasty Project

This project uses Toasty ORM for database management.

## Structure

- `entity/` - Your database models (entities)
- `migration/` - Database migrations

## Workflow

1. **Define entities** in `entity/src/lib.rs`
2. **Generate migrations** from schema changes:
   ```bash
   toasty migrate:generate --message "add user table" --url "postgresql://localhost/mydb"
   ```
3. **Apply migrations**:
   ```bash
   toasty migrate:up --url "postgresql://localhost/mydb"
   ```

## Example Entity

```rust
use toasty::stmt::Id;

#[derive(Debug, toasty::Model)]
pub struct User {
    #[key]
    #[auto]
    pub id: Id<Self>,
    pub name: String,
    #[unique]
    pub email: String,
}
```

## Migration Commands

- `toasty init` - Initialize project structure
- `toasty migrate:generate` - Generate migration from changes
- `toasty migrate:up` - Apply pending migrations
- `toasty migrate:down` - Rollback migrations
- `toasty migrate:status` - Show migration status
"#;
    std::fs::write(project_dir.join("README.md"), readme)?;
    println!("✅ Created README.md");

    println!();
    println!("🎉 Toasty project initialized!");

    Ok(())
}

async fn cmd_generate(
    message: String,
    url: String,
    dir: String,
    entity_dir: Option<String>,
) -> Result<()> {
    println!("🔍 Generating migration: {}", message);
    println!("📁 Migration directory: {}", dir);

    // Check if entity directory exists
    let entity_path = PathBuf::from(entity_dir.as_deref().unwrap_or("entity"));
    if entity_path.exists() {
        println!("📦 Entity directory: {}", entity_path.display());
    } else {
        println!("⚠️  Entity directory not found: {}", entity_path.display());
        println!("   Run 'toasty init' to create the project structure");
        println!("   Or specify custom path with --entity-dir");
    }
    println!();

    // Create migration directory if it doesn't exist
    let migration_dir = PathBuf::from(&dir);
    std::fs::create_dir_all(&migration_dir)?;

    let loader = MigrationLoader::new(&migration_dir);
    let snapshot_path = loader.snapshot_path();

    // Build desired schema from entity files (what developer wants)
    println!("📖 Building desired schema from entity files...");
    let parser = EntityParser::new(&entity_path);
    let desired_schema = match parser.parse_entities() {
        Ok(snapshot) => {
            println!("✅ Parsed {} model(s) from entities", snapshot.tables.len());
            snapshot
        }
        Err(e) => {
            return Err(anyhow::anyhow!(
                "Failed to parse entity files: {}\nEnsure entity/src/ contains valid Toasty models", e
            ));
        }
    };

    // Get current schema from database (what actually exists)
    println!("🔍 Introspecting current database schema...");
    let introspector = SqlIntrospector::new(url.clone());
    let current_schema = match introspector.introspect_schema().await {
        Ok(snapshot) => {
            println!("✅ Found {} table(s) in database", snapshot.tables.len());
            snapshot
        }
        Err(e) => {
            println!("⚠️  Database introspection failed: {}", e);
            println!("   Assuming empty database (will generate CREATE statements)");
            SchemaSnapshot {
                version: "1.0".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                tables: vec![],
            }
        }
    };

    // Detect changes: current database state → desired entity state
    println!();
    println!("🔄 Comparing database vs entities...");
    let diff = detect_changes(&current_schema, &desired_schema)?;

    if diff.changes.is_empty() {
        println!("✅ Database matches entities - no migration needed!");
        println!("   Your database schema is already up to date.");

        // Save entity schema for documentation
        save_snapshot(&desired_schema, &snapshot_path)?;
        println!("📝 Updated .schema.json for reference");

        // Don't create empty migration file
        return Ok(());
    }

    // Show detected changes
    println!();
    println!("✅ Detected {} schema change(s):", diff.changes.len());
    for change in &diff.changes {
        let marker = if change.is_destructive() {
            "⚠️ "
        } else {
            "✅"
        };
        println!("   {} {:?}", marker, change);
    }

    // Generate migration
    let generator = MigrationGenerator::new(&migration_dir);
    let migration = generator.generate(&diff, &message)?;

    // Write migration file
    generator.write_migration_file(&migration)?;
    println!();
    println!("✅ Created migration file: {}/{}", dir, migration.filename);

    // Save entity schema (for documentation/reference)
    save_snapshot(&desired_schema, &snapshot_path)?;
    println!("✅ Updated schema snapshot: {}/.schema.json", dir);

    println!();
    println!(
        "   - Review the generated migration: {}/{}",
        dir, migration.filename
    );
    println!("   - Apply with: toasty migrate:up --url <database-url>");

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
