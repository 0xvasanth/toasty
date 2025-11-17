# Toasty CLI

Command-line tool for managing Toasty ORM projects, entities, and database migrations.

## Installation

```bash
cargo install toasty-cli
```

Or run from the workspace:
```bash
cargo run -p toasty-cli -- <command>
```

## Commands

### `toasty init`

Initialize a new Toasty project with proper structure.

```bash
toasty init
# or
toasty init --dir my-project
```

**Creates:**
```
project/
├── entity/              ← Your database models
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs      ← Define models here
├── migration/           ← Migration files
│   └── .schema.json    ← Schema snapshot
└── README.md           ← Project guide
```

**Next steps after init:**
1. Add `entity/` to your workspace `Cargo.toml`
2. Define models in `entity/src/lib.rs`
3. Generate migrations with `toasty migrate:generate`

---

### `toasty migrate:generate`

Generate a migration from schema changes.

**With database introspection (recommended):**
```bash
toasty migrate:generate \
  --message "add user table" \
  --url "postgresql://localhost/mydb"
```

**With custom entity location:**
```bash
toasty migrate:generate \
  --message "add email field" \
  --url "postgresql://localhost/mydb" \
  --entity-dir custom/entities \
  --dir custom/migrations
```

**Without database (manual template):**
```bash
toasty migrate:generate --message "manual migration"
```

**What it does:**
1. Checks for entity directory
2. Loads previous schema snapshot
3. Introspects current database schema (if --url provided)
4. Detects changes (tables, columns, indexes)
5. Generates migration file with SQL DDL
6. Saves new snapshot

**Output:**
```
🔍 Generating migration: add user table
📁 Migration directory: migration
📦 Entity directory: entity

📸 Loading existing schema snapshot...
🔍 Introspecting PostgreSQL schema...
✅ Found 1 table(s)
✅ Detected 1 schema change(s):
   ✅ CreateTable(users)

✅ Created migration file: migration/20251117_120000_add_user_table.rs
✅ Updated schema snapshot: migration/.schema.json
```

---

### `toasty migrate:up`

Apply pending migrations to database.

```bash
toasty migrate:up --url "postgresql://localhost/mydb"

# With custom migration directory
toasty migrate:up --url "postgresql://localhost/mydb" --dir custom/migrations
```

**What it does:**
1. Connects to database
2. Checks migration tracking table
3. Loads migration files
4. Executes pending migrations
5. Tracks applied migrations

---

### `toasty migrate:down`

Rollback migrations.

```bash
# Rollback last migration
toasty migrate:down --url "postgresql://localhost/mydb"

# Rollback last 3 migrations
toasty migrate:down --url "postgresql://localhost/mydb" --count 3
```

---

### `toasty migrate:status`

Show migration status.

```bash
toasty migrate:status --url "postgresql://localhost/mydb"
```

**Output:**
```
📊 Migration Status
📁 Migration directory: migration

Found 3 migration file(s):

Version                      | Filename
---------------------------- | --------
20251117_120000_add_users    | 20251117_120000_add_users.rs
20251117_130000_add_email    | 20251117_130000_add_email.rs
20251117_140000_add_index    | 20251117_140000_add_index.rs
```

---

## Complete Workflow Example

### 1. Initialize Project

```bash
$ toasty init
✅ Created entity crate: entity/
✅ Created migration directory: migration/
✅ Created README.md
```

### 2. Add Entity to Workspace

`Cargo.toml`:
```toml
[workspace]
members = [
    "entity",
    # ... other crates
]
```

### 3. Define Models

`entity/src/lib.rs`:
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

### 4. Create Database

```bash
createdb myapp_dev  # PostgreSQL
```

### 5. Generate Initial Migration

```bash
toasty migrate:generate \
  --message "create users table" \
  --url "postgresql://localhost/myapp_dev"
```

### 6. Apply Migration

```bash
toasty migrate:up --url "postgresql://localhost/myapp_dev"
```

### 7. Modify Entity & Generate Migration

Edit `entity/src/lib.rs` to add a field:
```rust
pub struct User {
    // ... existing fields
    pub age: Option<i32>,  // NEW
}
```

Generate migration:
```bash
toasty migrate:generate \
  --message "add age to users" \
  --url "postgresql://localhost/myapp_dev"
```

---

## Database Support

### PostgreSQL
```bash
--url "postgresql://localhost/mydb"
--url "postgresql://user:pass@localhost:5432/mydb"
```

### SQLite
```bash
--url "sqlite:mydb.db"
--url "sqlite::memory:"
```

### MySQL
```bash
--url "mysql://localhost/mydb"
```

### MongoDB
```bash
--url "mongodb://localhost:27017/mydb"
```

---

## Directory Structure

### Standard Layout (created by `init`)
```
project/
├── entity/                    ← Models go here
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
├── migration/                 ← Migrations go here
│   ├── .schema.json          ← Schema snapshot
│   ├── 20251117_*.rs         ← Migration files
│   └── ...
└── README.md
```

### Custom Layout
```bash
# Use custom directories
toasty migrate:generate \
  --entity-dir custom/models \
  --dir custom/migrations \
  --message "my migration"
```

---

## Features

- ✅ Project initialization with standard structure
- ✅ Entity crate template with examples
- ✅ Migration directory with snapshots
- ✅ Database schema introspection (PostgreSQL, SQLite, MySQL)
- ✅ Automatic change detection
- ✅ Migration file generation
- ✅ Entity directory validation

---

## Troubleshooting

### "Entity directory not found"

Run `toasty init` first, or specify custom path:
```bash
toasty migrate:generate --entity-dir path/to/entities --message "..."
```

### "No schema changes detected"

Ensure:
1. You've modified your models in `entity/`
2. You're using `--url` for database introspection
3. The database exists and is accessible

### Empty migrations generated

Use `--url` parameter to enable automatic change detection:
```bash
toasty migrate:generate --message "..." --url "postgresql://..."
```

---

## Development

Build CLI:
```bash
cargo build -p toasty-cli --features postgresql
```

Run locally:
```bash
cargo run -p toasty-cli --features postgresql -- init
```

---

## License

MIT License - see LICENSE file for details.
