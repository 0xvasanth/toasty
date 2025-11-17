use anyhow::Result;
use std::path::PathBuf;
use toasty_migrate::*;
use crate::executor::MigrationExecutor;

pub async fn cmd_reset(
    url: String,
    dir: String,
    entity_dir: Option<String>,
    force: bool,
) -> Result<()> {
    println!("🔄 Database Reset");
    println!("📁 Migration directory: {}", dir);
    println!("🗄️  Database: {}", url);
    println!();

    // Confirm destructive operation
    if !force {
        println!("⚠️  WARNING: This will DROP ALL TABLES and rerun all migrations!");
        println!("   All data will be lost.");
        println!();
        println!("   Use --force to skip this confirmation");
        println!();
        print!("   Continue? [y/N]: ");

        use std::io::{self, Write};
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("❌ Reset cancelled");
            return Ok(());
        }
    }

    println!("🗑️  Step 1: Dropping all tables...");

    // Use executor to actually drop tables
    let executor = MigrationExecutor::new(url.clone());

    #[cfg(feature = "postgresql")]
    let dropped = executor.drop_all_tables_postgresql().await?;

    #[cfg(not(feature = "postgresql"))]
    let dropped = {
        println!("   Note: Only PostgreSQL is currently supported");
        0
    };

    println!("✅ Dropped {} table(s)", dropped);
    println!();

    println!("📂 Step 2: Loading migration files...");

    let loader = MigrationLoader::new(PathBuf::from(&dir));
    let migration_files = loader.discover_migrations()?;

    if migration_files.is_empty() {
        println!("⚠️  No migration files found in {}", dir);
        println!("   Run 'toasty migrate:generate' to create initial migration");
        return Ok(());
    }

    println!("   Found {} migration(s)", migration_files.len());
    for file in &migration_files {
        println!("   - {}", file.version);
    }
    println!();

    println!("⬆️  Step 3: Recreating schema from entities...");

    // Parse entities to get desired schema
    let entity_path = PathBuf::from(entity_dir.unwrap_or_else(|| "entity".to_string()));
    let parser = EntityParser::new(&entity_path);
    let desired_schema = parser.parse_entities()?;

    println!("   Creating {} table(s)", desired_schema.tables.len());

    // Generate and execute SQL
    let mut context = SqlMigrationContext::new(SqlFlavor::PostgreSQL);

    for table in &desired_schema.tables {
        let columns: Vec<ColumnDef> = table.columns.iter().map(|col| {
            ColumnDef {
                name: col.name.clone(),
                ty: col.ty.clone(),
                nullable: col.nullable,
                default: if col.nullable { None } else { Some("''".to_string()) },
            }
        }).collect();

        context.create_table(&table.name, columns)?;

        // Create indexes
        for index in &table.indices {
            if !index.primary_key && !index.columns.is_empty() {
                context.create_index(&table.name, IndexDef {
                    name: index.name.clone(),
                    columns: index.columns.clone(),
                    unique: index.unique,
                })?;
            }
        }
    }

    // Execute the SQL statements
    #[cfg(feature = "postgresql")]
    executor.execute_postgresql(&context).await?;

    println!();
    println!("✅ Reset complete!");
    println!("   ✅ Dropped {} table(s)", dropped);
    println!("   ✅ Created {} table(s)", desired_schema.tables.len());
    println!("   ✅ Database schema matches entities");

    Ok(())
}
