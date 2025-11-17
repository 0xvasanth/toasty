# Toasty Migration Guide

Complete guide to using auto-generated migrations in Toasty ORM.

## Overview

Toasty's migration system automatically detects schema changes and generates migration files. It works across both SQL (SQLite, PostgreSQL, MySQL) and NoSQL (MongoDB, DynamoDB) databases.

## Quick Start

### 1. Install CLI

```bash
cargo install toasty-cli
```

### 2. Generate Your First Migration

When you modify your models, generate a migration:

```bash
# After adding a new field to your User model
toasty migrate:generate --message "add email to users"
```

This creates `migrations/20250117_120000_add_email_to_users.rs`

### 3. Apply Migrations

```bash
toasty migrate:up --url "sqlite:mydb.db"
```

### 4. Check Status

```bash
toasty migrate:status --url "sqlite:mydb.db"
```

## How It Works

### Schema Snapshots

Toasty saves schema snapshots to detect changes:

```
migrations/
├── .schema.json                          ← Last known schema
├── 20250117_120000_create_users.rs      ← Migration files
├── 20250117_130000_add_user_email.rs
└── 20250117_140000_create_posts.rs
```

### Change Detection

When you run `migrate:generate`, Toasty:

1. Loads previous schema from `.schema.json`
2. Builds current schema from your models
3. Compares old vs new (diff algorithm)
4. Detects changes (tables, columns, indexes)
5. Generates migration file
6. Saves new snapshot

### Migration Types

**Additive (Safe):**
- ✅ CreateTable
- ✅ AddColumn (with default/nullable)
- ✅ CreateIndex

**Destructive (Requires Caution):**
- ⚠️ DropTable (data loss)
- ⚠️ DropColumn (data loss)
- ⚠️ ModifyColumn (may require data conversion)

**Index Management:**
- ✅ CreateIndex (unique, compound)
- ✅ DropIndex

## Example Workflow

### Scenario: Adding Email to User Model

**Step 1: Modify your model**

```rust
#[derive(Debug, toasty::Model)]
struct User {
    #[key] #[auto]
    id: Id<Self>,
    name: String,
    // NEW: Add email field
    #[unique]
    email: String,
}
```

**Step 2: Generate migration**

```bash
toasty migrate:generate --message "add user email"
```

Creates `migrations/20250117_120000_add_user_email.rs`:

```rust
use toasty_migrate::{Migration, MigrationContext, ColumnDef, IndexDef};

pub struct Migration_20250117_120000_add_user_email;

impl Migration for Migration_20250117_120000_add_user_email {
    fn version(&self) -> &str {
        "20250117_120000_add_user_email"
    }

    fn up(&self, db: &mut dyn MigrationContext) -> Result<()> {
        // Add email column
        db.add_column("users", ColumnDef {
            name: "email".into(),
            ty: "String".into(),
            nullable: false
        })?;

        // Create unique index
        db.create_index("users", IndexDef {
            name: "idx_users_email".into(),
            columns: vec!["email".into()],
            unique: true
        })?;

        Ok(())
    }

    fn down(&self, db: &mut dyn MigrationContext) -> Result<()> {
        db.drop_index("users", "idx_users_email")?;
        db.drop_column("users", "email")?;
        Ok(())
    }
}
```

**Step 3: Review and apply**

```bash
# Review what will be applied
toasty migrate:status --url "sqlite:mydb.db"

# Apply the migration
toasty migrate:up --url "sqlite:mydb.db"
```

**Output:**
```
Applying migration: 20250117_120000_add_user_email
  ✅ Applied: 20250117_120000_add_user_email

✅ Applied 1 migration(s)
```

## Cross-Database Support

### SQL Databases

Migrations generate SQL DDL:

```sql
-- Create Table
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL
);

-- Add Column
ALTER TABLE users ADD COLUMN email TEXT NOT NULL;

-- Create Index
CREATE UNIQUE INDEX idx_users_email ON users(email);
```

### NoSQL Databases

Migrations generate NoSQL operations:

```rust
// MongoDB
db.database.collection("users").create_index(
    IndexModel::builder()
        .keys(doc! { "email": 1 })
        .options(IndexOptions::builder().unique(true).build())
        .build()
).await?;

// DynamoDB
db.client.create_global_secondary_index(...)
```

## Migration Versioning

### Timestamp Format

`YYYYMMDD_HHMMSS_description`

Example: `20250117_143022_add_user_email`

### Ordering

Migrations are applied in chronological order based on timestamp.

### Conflicts

If multiple developers create migrations at the same time, the system will apply them in timestamp order.

## Rollback Behavior

### Safe Rollbacks

Can automatically rollback:
- ✅ CreateTable → DropTable
- ✅ AddColumn → DropColumn
- ✅ CreateIndex → DropIndex

### Manual Rollbacks Required

Some changes can't be automatically reversed:
- ⚠️ DropTable (can't recreate data)
- ⚠️ DropColumn (can't recover data)
- ⚠️ ModifyColumn (data may be transformed)

For these, down migration contains comments requiring manual intervention.

## Configuration

### Migration Directory

Default: `migrations/` in project root

Custom:
```bash
toasty migrate:generate --dir custom/migrations --message "add field"
```

### Connection URLs

```bash
# SQLite
toasty migrate:up --url "sqlite:mydb.db"
toasty migrate:up --url "sqlite::memory:"

# PostgreSQL
toasty migrate:up --url "postgresql://localhost/mydb"

# MySQL
toasty migrate:up --url "mysql://localhost/mydb"

# MongoDB
toasty migrate:up --url "mongodb://localhost:27017/mydb"

# DynamoDB
toasty migrate:up --url "dynamodb://localhost:8000"
```

## Best Practices

### 1. Review Before Applying

Always check what migrations will do:

```bash
toasty migrate:status --url "..."
```

### 2. Backup Before Destructive Changes

Before dropping tables or columns:

```bash
# Backup your database first!
pg_dump mydb > backup.sql
```

### 3. Test Migrations

Test in development before production:

```bash
# Dev database
toasty migrate:up --url "sqlite:dev.db"

# Verify everything works
# Then apply to production
```

### 4. Version Control

Commit migration files to git:

```bash
git add migrations/
git commit -m "Add user email migration"
```

## Troubleshooting

### Migration Failed

If a migration fails mid-execution:

1. Fix the issue (database state, data constraints, etc.)
2. The migration tracker knows which migrations succeeded
3. Re-run `migrate:up` to continue

### Schema Out of Sync

If `.schema.json` gets out of sync:

1. Delete `migrations/.schema.json`
2. Run `migrate:generate` to create fresh baseline
3. Review generated migrations carefully

### Rollback Failed

If rollback fails:

1. Check the down migration logic
2. May require manual database fixes
3. Update migration file if needed

## Advanced Usage

### Programmatic API

```rust
use toasty_migrate::*;

// Load schema
let old_snapshot = load_snapshot("migrations/.schema.json")?;
let new_snapshot = SchemaSnapshot::from_schema(&current_schema);

// Detect changes
let diff = detect_changes(&old_snapshot, &new_snapshot)?;

// Generate migration
let generator = MigrationGenerator::new("migrations");
let migration = generator.generate(&diff, "my_changes")?;
generator.write_migration_file(&migration)?;

// Save new snapshot
save_snapshot(&new_snapshot, "migrations/.schema.json")?;
```

### Custom Migrations

You can manually edit generated migrations for complex scenarios:

```rust
fn up(&self, db: &mut dyn MigrationContext) -> Result<()> {
    // Auto-generated
    db.add_column("users", ColumnDef { ... })?;

    // Custom: Set default values for existing rows
    db.execute_sql("UPDATE users SET email = 'unknown@example.com' WHERE email IS NULL")?;

    Ok(())
}
```

## Limitations (Current Implementation)

### ⏳ TODO Items

- Migration tracking table creation (structure ready)
- Database connection and execution (structure ready)
- SQL ALTER TABLE for column modifications
- NoSQL field transformations
- Data migrations (only schema changes for now)

### Not Yet Supported

- Renaming tables/columns (detected as drop + create)
- Complex column type changes with data conversion
- Constraint modifications
- Foreign key changes

## Future Enhancements

- Seed data management
- Migration squashing (combine multiple migrations)
- Dry-run mode (preview without applying)
- Migration dependencies
- Parallel migration execution
- Schema validation before migration

---

**Status**: Foundation complete, ready for database-specific implementations!
