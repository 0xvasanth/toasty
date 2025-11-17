# Migration Workflow Example

Complete example demonstrating Toasty's migration system with entities.

## Overview

This example shows:
- ✅ Entity crate structure (`entity/`)
- ✅ Multiple models with relationships
- ✅ Migration generation from database introspection
- ✅ Complete migration workflow
- ✅ Application using the entities

## Project Structure

```
migration-workflow/
├── Cargo.toml              ← Main application
├── src/
│   └── main.rs            ← Application code
├── entity/                 ← Entity crate (models)
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs         ← Model definitions
├── migration/              ← Migration files (generated)
│   ├── .schema.json       ← Schema snapshot
│   └── YYYYMMDD_*.rs      ← Migration files
└── README.md              ← This file
```

## Entities Defined

### User
- Fields: id, name, username (unique), email (unique)
- Relations: HasMany posts, HasMany user_roles

### Post
- Fields: id, user_id, title, content, published
- Relations: BelongsTo user

### Role
- Fields: id, name (unique), description
- Relations: HasMany user_roles

### UserRole (Junction Table)
- Fields: id, user_id, role_id
- Relations: BelongsTo user, BelongsTo role

## Complete Workflow

### Step 1: Initialize (Already Done)

This example already has the structure. For new projects:
```bash
toasty init
```

### Step 2: Set Up Database

**Using existing Docker container (recommended):**
```bash
# Use the running postgres container on port 5433
export DATABASE_URL="postgresql://postgres:postgres@localhost:5433/postgres"
```

**Or create new database:**
```bash
# PostgreSQL
createdb toasty_migration_example

# Or use Docker
docker run -d \
  -p 5432:5432 \
  -e POSTGRES_DB=toasty_migration_example \
  -e POSTGRES_PASSWORD=postgres \
  --name toasty-postgres \
  postgres:16-alpine
```

### Step 3: Run Application (Creates Schema)

```bash
# Set database URL (using Docker container on port 5433)
export DATABASE_URL="postgresql://postgres:postgres@localhost:5433/postgres"

# Run example (creates tables via reset_db)
cargo run --example migration-workflow --features postgresql
```

**Output:**
```
=== Toasty Migration Workflow Example ===
Connecting to: postgresql://localhost/toasty_migration_example
Database ready

Created: user=Alice, role=admin
```

### Step 4: Generate Initial Migration

Now that the database has schema, generate a migration from it:

```bash
cargo run -p toasty-cli --features postgresql -- migrate:generate \
  --message "initial schema" \
  --url "postgresql://postgres:postgres@localhost:5433/postgres" \
  --dir examples/migration-workflow/migration
```

**Output:**
```
🔍 Introspecting PostgreSQL schema...
✅ Found 4 table(s)
✅ Detected 4 schema change(s):
   ✅ CreateTable(users)
   ✅ CreateTable(posts)
   ✅ CreateTable(roles)
   ✅ CreateTable(user_roles)

✅ Created migration file: migration/20251117_120000_initial_schema.rs
✅ Updated schema snapshot: migration/.schema.json
```

### Step 5: Review Generated Migration

Check `migration/20251117_120000_initial_schema.rs`:

```rust
fn up(&self, db: &mut dyn MigrationContext) -> Result<()> {
    db.create_table("users", vec![...])?;
    db.create_table("posts", vec![...])?;
    db.create_table("roles", vec![...])?;
    db.create_table("user_roles", vec![...])?;
    Ok(())
}
```

### Step 6: Modify Entities

Edit `entity/src/lib.rs` to add a field:

```rust
pub struct User {
    // ... existing fields
    pub age: Option<i32>,  // NEW FIELD
}
```

### Step 7: Drop & Recreate Database

```bash
# Drop database
dropdb toasty_migration_example
createdb toasty_migration_example

# Run app again to create new schema
cargo run --example migration-workflow
```

### Step 8: Generate Migration for Changes

```bash
cargo run -p toasty-cli --features postgresql -- migrate:generate \
  --message "add age to users" \
  --url "postgresql://postgres:postgres@localhost/toasty_migration_example" \
  --dir examples/migration-workflow/migration
```

**Output:**
```
✅ Detected 1 schema change(s):
   ✅ AddColumn { table: "users", column: "age" }

✅ Created migration file: migration/20251117_130000_add_age_to_users.rs
```

### Step 9: Check Migration Status

```bash
cargo run -p toasty-cli --features postgresql -- migrate:status \
  --url "postgresql://postgres:postgres@localhost/toasty_migration_example" \
  --dir examples/migration-workflow/migration
```

**Output:**
```
Found 2 migration file(s):

Version                        | Filename
------------------------------ | --------
20251117_120000_initial_schema | 20251117_120000_initial_schema.rs
20251117_130000_add_age        | 20251117_130000_add_age.rs
```

## Migration Commands Reference

### Generate Migration
```bash
# From database introspection (recommended)
toasty migrate:generate \
  --message "description" \
  --url "postgresql://localhost/mydb" \
  --dir examples/migration-workflow/migration

# Manual template (no introspection)
toasty migrate:generate \
  --message "manual changes" \
  --dir examples/migration-workflow/migration
```

### Apply Migrations
```bash
toasty migrate:up \
  --url "postgresql://localhost/mydb" \
  --dir examples/migration-workflow/migration
```

### Rollback Migrations
```bash
toasty migrate:down \
  --url "postgresql://localhost/mydb" \
  --count 1 \
  --dir examples/migration-workflow/migration
```

### Check Status
```bash
toasty migrate:status \
  --url "postgresql://localhost/mydb" \
  --dir examples/migration-workflow/migration
```

## Development Workflow

### Typical Development Cycle

1. **Develop with reset_db()** (fast iteration)
   ```rust
   db.reset_db().await?;  // Destroys data but fast
   ```

2. **Before committing, generate migration**
   ```bash
   toasty migrate:generate --url "..." --message "add feature X"
   ```

3. **Commit entity changes + migration**
   ```bash
   git add entity/src/lib.rs migration/
   git commit -m "Add feature X"
   ```

4. **In production, use migrations**
   ```bash
   toasty migrate:up --url "..."
   ```

## Entity Best Practices

### Organization
- Keep all models in `entity/src/lib.rs`
- One file for easy tracking
- Clear model documentation
- Export models publicly

### Relationships
- Use `#[has_many]` for one-to-many
- Use `#[belongs_to]` with key references
- Junction tables for many-to-many

### Constraints
- `#[unique]` for unique constraints
- `#[index]` for indexed fields
- `#[key]` + `#[auto]` for auto-increment IDs

## Troubleshooting

### "No schema changes detected"
- Ensure database has current schema
- Run application first to create tables
- Check that --url connects successfully

### "Introspection failed"
- Verify database is running
- Check connection URL
- Ensure PostgreSQL feature is enabled: `--features postgresql`

### Migration not detecting new field
- Ensure database schema is updated (run app with reset_db)
- Check .schema.json has old state
- Verify field is in entity/src/lib.rs

## Database Connections

### PostgreSQL
```bash
# Local
postgresql://localhost/dbname

# With auth
postgresql://user:pass@localhost:5432/dbname

# Docker
postgresql://postgres:postgres@localhost:5432/dbname
```

### SQLite
```bash
sqlite:examples/migration-workflow/app.db
```

## Files

- `entity/` - Your models (version controlled)
- `migration/` - Generated migrations (version controlled)
- `migration/.schema.json` - Schema snapshot (version controlled)
- `src/main.rs` - Application code

## Next Steps

1. Run the example application
2. Try generating migrations
3. Modify entities and regenerate
4. See how migrations detect changes

This example demonstrates the complete migration workflow in a real project!
