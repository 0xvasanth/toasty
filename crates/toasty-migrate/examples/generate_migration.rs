use toasty_migrate::*;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Toasty Migration Generation Example ===\n");

    // Example: Simulate detecting schema changes
    // In real usage, old_snapshot would be loaded from .schema.json
    // and new_snapshot would be built from current models

    let old_snapshot = create_example_old_schema();
    let new_snapshot = create_example_new_schema();

    println!("📸 Comparing schema snapshots...");
    let diff = detect_changes(&old_snapshot, &new_snapshot)?;

    println!("\n🔍 Detected {} change(s):", diff.changes.len());
    for (i, change) in diff.changes.iter().enumerate() {
        let marker = if change.is_destructive() {
            "⚠️ "
        } else {
            "✅"
        };
        println!("  {}. {} {:?}", i + 1, marker, change);
    }

    // Generate migration
    println!("\n📝 Generating migration file...");
    let generator = MigrationGenerator::new("migrations");
    let migration = generator.generate(&diff, "add_user_email")?;

    println!("  Version: {}", migration.version);
    println!("  Filename: {}", migration.filename);
    println!("\n  Up statements:");
    for stmt in &migration.up_statements {
        println!("    {}", stmt);
    }
    println!("\n  Down statements:");
    for stmt in &migration.down_statements {
        println!("    {}", stmt);
    }

    // Write migration file
    println!("\n💾 Writing migration file...");
    generator.write_migration_file(&migration)?;
    println!("  ✅ Created: migrations/{}", migration.filename);

    // Save new snapshot
    println!("\n📸 Saving new schema snapshot...");
    save_snapshot(&new_snapshot, generator.migration_dir.join(".schema.json"))?;
    println!("  ✅ Saved: migrations/.schema.json");

    println!("\n🎉 Migration generated successfully!");

    Ok(())
}

fn create_example_old_schema() -> SchemaSnapshot {
    use toasty_migrate::snapshot::*;

    SchemaSnapshot {
        version: "1.0".to_string(),
        timestamp: "2025-01-17T00:00:00Z".to_string(),
        tables: vec![TableSnapshot {
            name: "users".to_string(),
            columns: vec![
                ColumnSnapshot {
                    name: "id".to_string(),
                    ty: "Id".to_string(),
                    nullable: false,
                },
                ColumnSnapshot {
                    name: "name".to_string(),
                    ty: "String".to_string(),
                    nullable: false,
                },
            ],
            indices: vec![IndexSnapshot {
                name: "pk_users".to_string(),
                columns: vec!["id".to_string()],
                unique: true,
                primary_key: true,
            }],
            primary_key: vec!["id".to_string()],
        }],
    }
}

fn create_example_new_schema() -> SchemaSnapshot {
    use toasty_migrate::snapshot::*;

    SchemaSnapshot {
        version: "1.0".to_string(),
        timestamp: "2025-01-17T01:00:00Z".to_string(),
        tables: vec![TableSnapshot {
            name: "users".to_string(),
            columns: vec![
                ColumnSnapshot {
                    name: "id".to_string(),
                    ty: "Id".to_string(),
                    nullable: false,
                },
                ColumnSnapshot {
                    name: "name".to_string(),
                    ty: "String".to_string(),
                    nullable: false,
                },
                // NEW: Email field added
                ColumnSnapshot {
                    name: "email".to_string(),
                    ty: "String".to_string(),
                    nullable: false,
                },
            ],
            indices: vec![
                IndexSnapshot {
                    name: "pk_users".to_string(),
                    columns: vec!["id".to_string()],
                    unique: true,
                    primary_key: true,
                },
                // NEW: Unique email index
                IndexSnapshot {
                    name: "idx_users_email".to_string(),
                    columns: vec!["email".to_string()],
                    unique: true,
                    primary_key: false,
                },
            ],
            primary_key: vec!["id".to_string()],
        }],
    }
}
