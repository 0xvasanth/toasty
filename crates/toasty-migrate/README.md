# toasty-migrate

Auto-generated database migration system for Toasty ORM.

## Features

- ✅ Auto-detect schema changes by comparing snapshots
- ✅ Generate migration files automatically
- ✅ Support for both SQL and NoSQL databases
- ✅ Classify changes as additive/destructive/modificative
- ✅ Rollback support with down migrations
- ⏳ Migration tracking (TODO)
- ⏳ Automatic execution (TODO)

## How It Works

### 1. Schema Snapshots

The system saves schema snapshots to detect changes:

```rust
use toasty_migrate::{SchemaSnapshot, save_snapshot, load_snapshot};

// Create snapshot from current schema
let snapshot = SchemaSnapshot::from_schema(&schema);
save_snapshot(&snapshot, "migrations/.schema.json")?;

// Load previous snapshot
let old_snapshot = load_snapshot("migrations/.schema.json")?;
```

### 2. Change Detection

Compare schemas to detect changes:

```rust
use toasty_migrate::detect_changes;

let diff = detect_changes(&old_snapshot, &new_snapshot)?;

for change in &diff.changes {
    if change.is_destructive() {
        println!("⚠️  Destructive: {:?}", change);
    } else {
        println!("✅ Additive: {:?}", change);
    }
}
```

### 3. Migration Generation

Generate migration files from detected changes:

```rust
use toasty_migrate::MigrationGenerator;

let generator = MigrationGenerator::new("migrations");
let migration = generator.generate(&diff, "add_user_email")?;
generator.write_migration_file(&migration)?;
```

This creates `migrations/20250117_120000_add_user_email.rs` with:

```rust
pub struct Migration_20250117_120000_add_user_email;

impl Migration for Migration_20250117_120000_add_user_email {
    fn version(&self) -> &str {
        "20250117_120000_add_user_email"
    }

    fn up(&self, db: &mut dyn MigrationContext) -> Result<()> {
        db.add_column("users", ColumnDef {
            name: "email".into(),
            ty: "String".into(),
            nullable: false
        })?;
        Ok(())
    }

    fn down(&self, db: &mut dyn MigrationContext) -> Result<()> {
        db.drop_column("users", "email")?;
        Ok(())
    }
}
```

## Detected Change Types

| Change Type | SQL Example | NoSQL Equivalent |
|-------------|-------------|------------------|
| CreateTable | `CREATE TABLE users (...)` | Create collection + indexes |
| DropTable | `DROP TABLE users` | Drop collection |
| AddColumn | `ALTER TABLE users ADD COLUMN email VARCHAR(255)` | Add field to documents |
| DropColumn | `ALTER TABLE users DROP COLUMN email` | Remove field from documents |
| ModifyColumn | `ALTER TABLE users ALTER COLUMN age TYPE INT` | Data transformation |
| CreateIndex | `CREATE INDEX idx_email ON users(email)` | `collection.create_index()` |
| DropIndex | `DROP INDEX idx_email` | `collection.drop_index()` |

## Usage with CLI

```bash
# Generate a migration
toasty migrate:generate --message "add user email field"

# Apply pending migrations
toasty migrate:up --url "sqlite::memory:"

# Rollback last migration
toasty migrate:down --url "sqlite::memory:" --count 1

# Check migration status
toasty migrate:status --url "sqlite::memory:"
```

## Architecture

```
Schema Changes Detected
         ↓
  Schema Diff Created
         ↓
  Migration File Generated
         ↓
  Migration Applied to Database
         ↓
  Tracked in _toasty_migrations
```

## Current Implementation Status

### ✅ Implemented
- Schema snapshot serialization
- Change detection (7 change types)
- Migration file generation
- CLI structure with all commands

### ⏳ TODO
- Migration tracking table (`_toasty_migrations`)
- Migration runner implementation
- Database-specific MigrationContext implementations
- SQL DDL generation for ALTER TABLE
- NoSQL-specific migration operations
- Integration tests

## Design Principles

1. **Cross-Database**: Works with SQL and NoSQL databases
2. **Type-Safe**: Schema changes detected at compile time
3. **Auto-Generated**: No manual migration writing needed
4. **Rollback Support**: Safe reversible migrations
5. **Timestamp Versioning**: Clear migration ordering

## Future Enhancements

- Data migrations (not just schema)
- Seed data management
- Migration squashing
- Schema validation before migration
- Dry-run mode
- Migration dependencies

## License

MIT License - see LICENSE file for details.
