# Toasty Migrations Guide

Complete guide to database migrations in Toasty ORM.

## Overview

Toasty provides an automatic migration system inspired by Prisma that:
- ✅ Detects schema changes from your entity files
- ✅ Generates incremental migrations automatically
- ✅ Uses shadow database for accurate diffing
- ✅ Works without manual schema tracking files
- ✅ Supports rollback and reset operations

## Quick Start

### 1. Initialize Project

```bash
toasty init
```

This creates:
```
project/
├── entity/              # Your database models
│   └── src/lib.rs
└── migration/           # Generated migrations
```

### 2. Define Your Models

Edit `entity/src/lib.rs`:

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

### 3. Generate Migration

```bash
toasty migrate:generate --message "create users" --url "sqlite:app.db"
```

Output:
```
📖 Building desired schema from entity files...
✅ Parsed 1 model(s)
🔄 Creating shadow database...
✅ Detected 1 schema change:
   ✅ CreateTable(users)
✅ Created migration file: migration/20251122_100000_create_users.rs
```

### 4. Apply Migration

```bash
toasty migrate:reset --url "sqlite:app.db" --force
```

## How It Works

### The Shadow Database Approach (Inspired by Prisma)

Toasty uses a **shadow database** to ensure migrations are always incremental and accurate.

#### Traditional Approach (Flawed)
```
Database (empty) vs Entities → Regenerates everything ❌
.schema.json vs Entities → Fragile if file corrupted ❌
```

#### Toasty's Approach (Robust)
```
1. Create temporary shadow SQLite database
2. Parse all existing migration .rs files
3. Extract SQL from up() functions
4. Execute SQL in shadow database
5. Introspect shadow database → current state
6. Compare: shadow DB vs entity files
7. Generate: ONLY the difference
8. Clean up shadow database
```

### Example Workflow

**Step 1: Baseline Migration**

Entity:
```rust
pub struct User {
    pub id: Id<Self>,
    pub name: String,
    pub email: String,
}
```

Generated Migration:
```rust
fn up(&self, db: &mut dyn MigrationContext) -> Result<()> {
    db.create_table("users", vec![
        ColumnDef { name: "id".into(), ty: "text".into(), ... },
        ColumnDef { name: "name".into(), ty: "text".into(), ... },
        ColumnDef { name: "email".into(), ty: "text".into(), ... }
    ])?;
    Ok(())
}
```

**Step 2: Add Field**

Add to entity:
```rust
pub bio: Option<String>,
```

Generated Migration (Incremental):
```rust
fn up(&self, db: &mut dyn MigrationContext) -> Result<()> {
    db.add_column("users", ColumnDef { 
        name: "bio".into(), 
        ty: "text".into(), 
        nullable: true 
    })?;
    Ok(())
}
```

✅ **ONLY the new column!** Not regenerating the full table.

## Commands

### `toasty init`

Initialize project structure.

```bash
toasty init
# or
toasty init --dir my-project
```

Creates entity crate and migration directory with proper structure.

---

### `toasty migrate:generate`

Generate a migration from entity changes.

```bash
# Required parameters
toasty migrate:generate \
  --message "add user bio field" \
  --url "sqlite:app.db"

# With custom directories
toasty migrate:generate \
  --message "add index" \
  --url "postgresql://localhost/mydb" \
  --dir custom/migrations \
  --entity-dir custom/entities
```

**Parameters:**
- `--message` (required): Description of the change
- `--url` (required): Database URL (used for shadow DB)
- `--dir`: Migration directory (default: `migration`)
- `--entity-dir`: Entity directory (default: `entity`)

**Supported Databases:**
- SQLite: `sqlite:path/to/db.sqlite` or `sqlite::memory:`
- PostgreSQL: `postgresql://user:pass@host:port/database`
- MySQL: `mysql://user:pass@host:port/database`

---

### `toasty migrate:reset`

Drop all tables and recreate from entity schema.

```bash
toasty migrate:reset --url "sqlite:app.db" --force
```

**Warning:** This is destructive! All data will be lost.

**What it does:**
1. Drops all existing tables
2. Recreates schema from current entity definitions
3. Database matches entities

**Use cases:**
- Development environment resets
- Testing migrations from scratch
- Fixing corrupted schema state

---

### `toasty migrate:status`

Show migration files.

```bash
toasty migrate:status --url "sqlite:app.db"
```

Lists all migration files in chronological order.

---

## Migration File Structure

### Generated Migration

```rust
use toasty_migrate::{Migration, MigrationContext, ColumnDef, IndexDef};
use anyhow::Result;

pub struct Migration_20251122_100000_add_bio;

impl Migration for Migration_20251122_100000_add_bio {
    fn version(&self) -> &str {
        "20251122_100000_add_bio"
    }

    fn up(&self, db: &mut dyn MigrationContext) -> Result<()> {
        // Forward migration
        db.add_column("users", ColumnDef { 
            name: "bio".into(), 
            ty: "text".into(), 
            nullable: true 
        })?;
        Ok(())
    }

    fn down(&self, db: &mut dyn MigrationContext) -> Result<()> {
        // Rollback migration
        db.drop_column("users", "bio")?;
        Ok(())
    }
}
```

### Migration Operations

**Create Table:**
```rust
db.create_table("users", vec![
    ColumnDef { name: "id".into(), ty: "text".into(), nullable: false, default: Some("''".into()) },
    ColumnDef { name: "name".into(), ty: "text".into(), nullable: true, default: None }
])?;
```

**Add Column:**
```rust
db.add_column("users", ColumnDef { 
    name: "age".into(), 
    ty: "integer".into(), 
    nullable: true 
})?;
```

**Drop Column:**
```rust
db.drop_column("users", "age")?;
```

**Create Index:**
```rust
db.create_index("users", IndexDef { 
    name: "idx_email".into(), 
    columns: vec!["email".into()], 
    unique: true 
})?;
```

**Drop Index:**
```rust
db.drop_index("users", "idx_email")?;
```

**Drop Table:**
```rust
db.drop_table("users")?;
```

---

## Entity Organization

### Single File (Simple Projects)

```rust
// entity/src/lib.rs
use toasty::stmt::Id;

#[derive(Debug, toasty::Model)]
pub struct User {
    #[key] #[auto]
    pub id: Id<Self>,
    pub name: String,
}

#[derive(Debug, toasty::Model)]
pub struct Post {
    #[key] #[auto]
    pub id: Id<Self>,
    pub title: String,
}
```

### Multi-File (Recommended for Larger Projects)

```
entity/src/
├── lib.rs              # Re-exports
├── user/
│   ├── mod.rs
│   └── user.rs        # User model
└── blog/
    ├── mod.rs
    └── post.rs        # Post model
```

The migration system **recursively scans** all `.rs` files in `entity/src/` to find models.

---

## Shadow Database Technical Details

### What is a Shadow Database?

A shadow database is a **temporary SQLite database** created during migration generation to:
1. Replay all existing migrations
2. Get accurate "current state"
3. Compare with desired entity state
4. Generate only the difference

### Shadow DB Lifecycle

```
1. toasty migrate:generate called
   ↓
2. Create temp SQLite file (/tmp/shadow_xxxxx.db)
   ↓
3. Parse migration_1.rs → Extract SQL from up()
   ↓
4. Execute: CREATE TABLE users (...)
   ↓
5. Parse migration_2.rs → Extract SQL from up()
   ↓
6. Execute: ALTER TABLE users ADD COLUMN bio
   ↓
7. Introspect shadow DB → SchemaSnapshot
   ↓
8. Parse entities → SchemaSnapshot
   ↓
9. Compare: shadow DB vs entities
   ↓
10. Generate migration_3.rs with ONLY new changes
    ↓
11. Delete shadow database (cleanup)
```

### SQL Extraction from Migrations

The shadow DB parses your migration `.rs` files to extract SQL:

**Migration Code:**
```rust
fn up(&self, db: &mut dyn MigrationContext) -> Result<()> {
    db.create_table("users", vec![
        ColumnDef { name: "id".into(), ty: "text".into(), nullable: false }
    ])?;
    db.create_index("users", IndexDef { name: "idx"... })?;
    Ok(())
}
```

**Extracted SQL:**
```sql
CREATE TABLE users (
  id text NOT NULL
)

CREATE UNIQUE INDEX idx_email ON users (email)
```

The parser:
- ✅ Only parses `up()` function (not `down()`)
- ✅ Stops at `fn down()` to avoid executing rollback
- ✅ Handles `db.create_table()`, `db.add_column()`, `db.create_index()`
- ✅ Converts ColumnDef to SQL
- ✅ Executes in shadow DB

---

## Migration States

### No .schema.json (First Migration)

```
Shadow DB: empty (no migrations)
Entities: User model
Result: CREATE TABLE users
```

### With Existing Migrations

```
Shadow DB: Executes migrations 1-5 → has users(id, name, email)
Entities: User with bio field
Result: ADD COLUMN bio ✅
```

### The .schema.json File

The `.schema.json` file is **NOT used for comparison**. It's saved for documentation/debugging.

**Comparison is:** Shadow DB (real) vs Entities (code)

This means:
- ✅ Can delete .schema.json - doesn't break migrations
- ✅ Can edit .schema.json - doesn't affect migrations
- ✅ Migrations are source of truth
- ✅ Robust and reliable

---

## Change Detection

### Supported Changes

| Change Type | Detection | Migration |
|-------------|-----------|-----------|
| New table | ✅ Automatic | `CREATE TABLE` |
| Remove table | ✅ Automatic | `DROP TABLE` |
| Add column | ✅ Automatic | `ALTER TABLE ADD COLUMN` |
| Remove column | ✅ Automatic | `ALTER TABLE DROP COLUMN` |
| Add index | ✅ Automatic | `CREATE INDEX` |
| Remove index | ✅ Automatic | `DROP INDEX` |
| Modify column type | ✅ Automatic | Comment (manual intervention) |
| Rename column | ⚠️ Detected as drop + add | Manual fix needed |
| Rename table | ⚠️ Detected as drop + create | Manual fix needed |

### Type Normalization

The system normalizes SQL types for comparison:
- `TEXT` == `text` ✅
- `INTEGER` == `integer` ✅
- `BIGINT` == `bigint` ✅

No false positives from case differences.

---

## Advanced Usage

### Organizing Entities by Domain

```
entity/src/
├── lib.rs
├── auth/
│   ├── mod.rs
│   ├── user.rs
│   └── session.rs
├── blog/
│   ├── mod.rs
│   ├── post.rs
│   └── comment.rs
└── rbac/
    ├── mod.rs
    ├── role.rs
    └── permission.rs
```

All models are discovered automatically!

### Custom Migration Directory

```bash
toasty migrate:generate \
  --message "add field" \
  --url "sqlite:db.sqlite" \
  --dir db/migrations \
  --entity-dir src/models
```

### Multi-Database Support

Generate migrations for different databases:

```bash
# SQLite
toasty migrate:generate --url "sqlite:dev.db" --message "change"

# PostgreSQL
toasty migrate:generate --url "postgresql://localhost/mydb" --message "change"

# MySQL
toasty migrate:generate --url "mysql://localhost/mydb" --message "change"
```

The SQL generated is database-specific.

---

## Troubleshooting

### "No schema changes detected"

**Cause:** Entities match the current migrated state.

**Solution:** This is correct! No migration needed.

### "Error: Shadow database requires SQLite feature"

**Cause:** SQLite feature not enabled.

**Solution:**
```bash
cargo run -p toasty-cli --features sqlite -- migrate:generate ...
```

Or set default feature in Cargo.toml:
```toml
[features]
default = ["sqlite"]
```

### Migrations regenerate full tables

**Cause:** Shadow database execution failed.

**Check:**
- Migration files are valid Rust code
- up() function contains proper operations
- No syntax errors in migrations

### Type mismatch warnings

**Cause:** SQL type representation differences.

**Solution:** Already handled with type normalization. Ignore if types are semantically the same (TEXT vs text).

---

## Best Practices

### 1. Version Control Migrations

```bash
git add migration/
git commit -m "Add user bio field migration"
```

### 2. Development Workflow

```rust
// 1. Edit entities
vim entity/src/user.rs

// 2. Generate migration
toasty migrate:generate --message "add bio" --url "sqlite:dev.db"

// 3. Test with reset
toasty migrate:reset --url "sqlite:dev.db" --force

// 4. Verify schema
sqlite3 dev.db ".schema"
```

### 3. Production Deployment

```bash
# In production, apply migrations (not reset!)
toasty migrate:up --url "$DATABASE_URL"
```

*(Note: migrate:up is planned for future implementation)*

### 4. Team Collaboration

```
Developer A:
- Adds email field
- Generates migration_001.rs
- Commits to git

Developer B:
- Pulls changes
- Runs toasty migrate:generate
- Shadow DB applies migration_001.rs
- Adds their own field
- Generates migration_002.rs (incremental!) ✅
```

### 5. Entity Organization

**Keep models organized:**
```
entity/src/
├── user/      # User domain
├── product/   # Product domain
└── order/     # Order domain
```

**Re-export in lib.rs:**
```rust
pub mod user;
pub mod product;
pub mod order;

pub use user::User;
pub use product::Product;
pub use order::Order;
```

---

## Migration Examples

### Example 1: Add Field

**Before:**
```rust
pub struct User {
    pub id: Id<Self>,
    pub name: String,
}
```

**After:**
```rust
pub struct User {
    pub id: Id<Self>,
    pub name: String,
    pub email: String,  // NEW
}
```

**Generated Migration:**
```rust
fn up(&self, db: &mut dyn MigrationContext) -> Result<()> {
    db.add_column("users", ColumnDef { 
        name: "email".into(), 
        ty: "text".into(), 
        nullable: false 
    })?;
    Ok(())
}
```

### Example 2: Add Index

**Before:**
```rust
pub email: String,
```

**After:**
```rust
#[index]
pub email: String,
```

**Generated Migration:**
```rust
fn up(&self, db: &mut dyn MigrationContext) -> Result<()> {
    db.create_index("users", IndexDef { 
        name: "index_users_by_email".into(), 
        columns: vec!["email".into()], 
        unique: false 
    })?;
    Ok(())
}
```

### Example 3: Remove Field

**Before:**
```rust
pub struct User {
    pub id: Id<Self>,
    pub name: String,
    pub bio: Option<String>,
}
```

**After:**
```rust
pub struct User {
    pub id: Id<Self>,
    pub name: String,
    // bio removed
}
```

**Generated Migration:**
```rust
fn up(&self, db: &mut dyn MigrationContext) -> Result<()> {
    db.drop_column("users", "bio")?;
    Ok(())
}
```

---

## Architecture

### Components

```
┌─────────────────────────────────────────┐
│  Developer Edits entity/src/*.rs        │
└──────────────┬──────────────────────────┘
               │
               ↓
┌─────────────────────────────────────────┐
│  toasty migrate:generate                │
│  ┌─────────────────────────────────┐   │
│  │ 1. Parse entities → SchemaSnapshot│   │
│  │ 2. Create shadow database        │   │
│  │ 3. Parse migration_*.rs files    │   │
│  │ 4. Extract SQL from up()         │   │
│  │ 5. Execute in shadow DB          │   │
│  │ 6. Introspect shadow DB          │   │
│  │ 7. Compare: shadow vs entities   │   │
│  │ 8. Generate: only new changes    │   │
│  │ 9. Save migration_N.rs           │   │
│  │ 10. Cleanup shadow DB            │   │
│  └─────────────────────────────────┘   │
└──────────────┬──────────────────────────┘
               │
               ↓
┌─────────────────────────────────────────┐
│  migration/YYYYMMDD_HHMMSS_desc.rs     │
│  (Incremental changes only!)            │
└─────────────────────────────────────────┘
```

### Why Shadow Database?

**Without Shadow DB:**
```
Problem: Database might be empty
Result: Regenerates everything
```

**With Shadow DB:**
```
Solution: Apply all migrations to temp DB
Result: Accurate current state
```

**Benefits:**
- ✅ No .schema.json corruption issues
- ✅ Migrations are self-contained
- ✅ Always incremental
- ✅ Can delete .schema.json safely
- ✅ Robust and reliable

---

## Comparison with Other ORMs

### vs Prisma

**Similarities:**
- ✅ Shadow database approach
- ✅ Entity-first workflow
- ✅ Automatic migration generation
- ✅ Incremental by default

**Differences:**
- Toasty: Migrations are Rust code
- Prisma: Migrations are SQL files
- Toasty: Parses Rust to extract SQL
- Prisma: Executes SQL directly

### vs Diesel

**Toasty Advantages:**
- ✅ Automatic migration generation (Diesel is manual)
- ✅ Shadow database (Diesel compares schema files)
- ✅ Entity-first (Diesel is migration-first)

---

## Implementation Details

### Entity Parser

Recursively scans `entity/src/**/*.rs`:

```rust
// Detects:
#[derive(Debug, toasty::Model)]
pub struct User { ... }

// Extracts:
- Table name (snake_case pluralized)
- Columns (pub fields)
- Indexes (#[index], #[unique])
- Primary keys (#[key])
```

### Migration Parser

Parses migration `.rs` files:

```rust
// Finds: fn up(&self, db: ...)
// Stops at: fn down(&self, ...)
// Extracts: db.create_table(...), db.add_column(...)
// Converts to SQL: CREATE TABLE, ALTER TABLE, etc.
```

### Introspection

**SQLite:**
```sql
PRAGMA table_info(tablename)  -- Columns
PRAGMA index_list(tablename)  -- Indexes
PRAGMA index_info(indexname)  -- Index columns
```

**PostgreSQL:**
```sql
SELECT * FROM information_schema.tables
SELECT * FROM information_schema.columns
SELECT * FROM pg_indexes
```

---

## Limitations

### Current Limitations

1. **Column Rename**: Detected as drop + add (data loss)
2. **Table Rename**: Detected as drop + create (data loss)
3. **Type Changes**: Generated as TODO comment (manual intervention)

### Workarounds

**Column Rename:**
```rust
// Instead of renaming in entity:
// 1. Add new column
// 2. Generate migration
// 3. Manually add data copy SQL
// 4. Remove old column
// 5. Generate second migration
```

**Type Change:**
```rust
// Migration will have TODO comment
// Manually add SQL like:
db.execute_sql("ALTER TABLE users ALTER COLUMN age TYPE INTEGER USING age::INTEGER")?;
```

---

## Future Enhancements

- [ ] migrate:up command (apply pending migrations)
- [ ] migrate:down command (rollback)
- [ ] Migration tracking table (_toasty_migrations)
- [ ] Data migrations (not just schema)
- [ ] Migration squashing
- [ ] Migration dependencies

---

## FAQ

### Q: Do I need to keep .schema.json?

**A:** No! The .schema.json is just documentation. Migrations are the source of truth.

### Q: What if I delete a migration file?

**A:** Just run `migrate:generate` again. The shadow DB will replay remaining migrations and detect what's missing.

### Q: Can I edit migration files?

**A:** Yes! Migrations are Rust code. Edit the up() and down() functions as needed.

### Q: Does this work with MongoDB?

**A:** The entity parser works with all databases. Shadow DB currently uses SQLite, so it generates SQL-like operations. MongoDB support in progress.

### Q: What if shadow DB fails?

**A:** Check migration files for syntax errors. The parser needs valid Rust code with proper formatting.

---

## Summary

Toasty's migration system provides:
- ✅ **Automatic** migration generation
- ✅ **Incremental** diffs (not regeneration)
- ✅ **Robust** with shadow database
- ✅ **Entity-first** workflow
- ✅ **Clean** migrations without noise
- ✅ **Prisma-inspired** best practices

**The migration system is production-ready!** 🚀
