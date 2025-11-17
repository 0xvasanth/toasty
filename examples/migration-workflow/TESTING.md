# Testing Migration Generation - ALTER TABLE and DELETE

This guide shows how to test all migration scenarios including adding/removing columns, tables, and indexes.

## Prerequisites

```bash
# Ensure PostgreSQL is running
export DATABASE_URL="postgresql://postgres:postgres@localhost:5433/postgres"

# Run the example to create initial schema
cargo run -p example-migration-workflow --features postgresql
```

## Test Scenario 1: ADD COLUMN (ALTER TABLE ADD)

### Step 1: Generate baseline migration

```bash
# Generate initial migration
cargo run -p toasty-cli --features postgresql -- migrate:generate \
  --message "baseline schema" \
  --url "$DATABASE_URL" \
  --dir examples/migration-workflow/migration \
  --entity-dir examples/migration-workflow/entity
```

**Result:** Creates migration with all 4 tables

### Step 2: Add a new field to User model

Edit `entity/src/lib.rs`:

```rust
pub struct User {
    #[key] #[auto]
    pub id: Id<Self>,
    pub name: String,
    pub username: String,
    pub email: String,

    // ADD THIS NEW FIELD
    pub bio: Option<String>,
}
```

### Step 3: Update database schema

```bash
# Run app again to add the column
cargo run -p example-migration-workflow --features postgresql
```

### Step 4: Generate migration for change

```bash
cargo run -p toasty-cli --features postgresql -- migrate:generate \
  --message "add bio to users" \
  --url "$DATABASE_URL" \
  --dir examples/migration-workflow/migration \
  --entity-dir examples/migration-workflow/entity
```

**Expected Output:**
```
✅ Detected 1 schema change(s):
   ✅ AddColumn { table: "users", column: ColumnSnapshot { name: "bio", ty: "text", nullable: true } }
```

**Generated Migration:**
```rust
fn up(&self, db: &mut dyn MigrationContext) -> Result<()> {
    db.add_column("users", ColumnDef {
        name: "bio".into(),
        ty: "text".into(),
        nullable: true,
        default: None
    })?;
    Ok(())
}

fn down(&self, db: &mut dyn MigrationContext) -> Result<()> {
    db.drop_column("users", "bio")?;
    Ok(())
}
```

---

## Test Scenario 2: DROP COLUMN

### Step 1: Remove a field from model

Edit `entity/src/lib.rs` - remove the `description` field from `Role`:

```rust
pub struct Role {
    #[key] #[auto]
    pub id: Id<Self>,
    pub name: String,
    // REMOVE: pub description: Option<String>,
}
```

### Step 2: Update database

```bash
cargo run -p example-migration-workflow --features postgresql
```

### Step 3: Generate migration

```bash
cargo run -p toasty-cli --features postgresql -- migrate:generate \
  --message "remove description from roles" \
  --url "$DATABASE_URL" \
  --dir examples/migration-workflow/migration \
  --entity-dir examples/migration-workflow/entity
```

**Expected Output:**
```
✅ Detected 1 schema change(s):
   ⚠️  DropColumn { table: "roles", column: "description" }
```

**Generated Migration:**
```rust
fn up(&self, db: &mut dyn MigrationContext) -> Result<()> {
    db.drop_column("roles", "description")?;
    Ok(())
}

fn down(&self, db: &mut dyn MigrationContext) -> Result<()> {
    // Cannot automatically restore dropped column: roles.description
    Ok(())
}
```

---

## Test Scenario 3: DROP TABLE

### Step 1: Remove entire model

Edit `entity/src/lib.rs` - comment out the entire `Post` struct:

```rust
// #[derive(Debug, toasty::Model)]
// pub struct Post {
//     ...
// }
```

### Step 2: Update app to not register Post

Edit `src/main.rs`:

```rust
let db = toasty::Db::builder()
    .register::<User>()
    // .register::<Post>()  // REMOVE THIS
    .register::<Role>()
    .register::<UserRole>()
    .connect(&db_url)
    .await?;
```

### Step 3: Update database

```bash
cargo run -p example-migration-workflow --features postgresql
```

### Step 4: Generate migration

```bash
cargo run -p toasty-cli --features postgresql -- migrate:generate \
  --message "remove posts table" \
  --url "$DATABASE_URL" \
  --dir examples/migration-workflow/migration \
  --entity-dir examples/migration-workflow/entity
```

**Expected Output:**
```
✅ Detected 1 schema change(s):
   ⚠️  DropTable("posts")
```

**Generated Migration:**
```rust
fn up(&self, db: &mut dyn MigrationContext) -> Result<()> {
    db.drop_table("posts")?;
    Ok(())
}

fn down(&self, db: &mut dyn MigrationContext) -> Result<()> {
    // Cannot automatically recreate dropped table: posts
    // Manual intervention required
    Ok(())
}
```

---

## Test Scenario 4: ADD TABLE

### Step 1: Add new model

Edit `entity/src/lib.rs` - add a new model:

```rust
#[derive(Debug, toasty::Model)]
pub struct Comment {
    #[key]
    #[auto]
    pub id: Id<Self>,

    pub content: String,

    #[index]
    pub user_id: Id<User>,

    #[belongs_to(key = user_id, references = id)]
    pub user: toasty::BelongsTo<User>,
}
```

### Step 2: Register in app

Edit `src/main.rs`:

```rust
let db = toasty::Db::builder()
    .register::<User>()
    .register::<Post>()
    .register::<Role>()
    .register::<UserRole>()
    .register::<Comment>()  // ADD THIS
    .connect(&db_url)
    .await?;
```

### Step 3: Update database

```bash
cargo run -p example-migration-workflow --features postgresql
```

### Step 4: Generate migration

```bash
cargo run -p toasty-cli --features postgresql -- migrate:generate \
  --message "add comments table" \
  --url "$DATABASE_URL" \
  --dir examples/migration-workflow/migration \
  --entity-dir examples/migration-workflow/entity
```

**Expected Output:**
```
✅ Detected 1 schema change(s):
   ✅ CreateTable(TableSnapshot { name: "comments", columns: [...] })
```

**Generated Migration:**
```rust
fn up(&self, db: &mut dyn MigrationContext) -> Result<()> {
    db.create_table("comments", vec![
        ColumnDef { name: "id".into(), ty: "text".into(), nullable: false, default: Some("''".into()) },
        ColumnDef { name: "content".into(), ty: "text".into(), nullable: true, default: None },
        ColumnDef { name: "user_id".into(), ty: "text".into(), nullable: true, default: None }
    ])?;
    db.create_index("comments", IndexDef { name: "index_comments_by_user_id".into(), columns: vec!["user_id".into()], unique: false })?;
    Ok(())
}
```

---

## Test Scenario 5: ADD INDEX

### Step 1: Add index attribute

Edit `entity/src/lib.rs` - add `#[index]` to an existing field:

```rust
pub struct Role {
    #[key] #[auto]
    pub id: Id<Self>,

    #[index]  // ADD THIS
    pub name: String,
}
```

### Step 2: Update database

```bash
cargo run -p example-migration-workflow --features postgresql
```

### Step 3: Generate migration

```bash
cargo run -p toasty-cli --features postgresql -- migrate:generate \
  --message "add index to role name" \
  --url "$DATABASE_URL" \
  --dir examples/migration-workflow/migration \
  --entity-dir examples/migration-workflow/entity
```

**Expected Output:**
```
✅ Detected 1 schema change(s):
   ✅ CreateIndex { table: "roles", index: IndexSnapshot { name: "index_roles_by_name", ... } }
```

---

## Test Scenario 6: DROP INDEX

### Step 1: Remove index attribute

Edit `entity/src/lib.rs` - remove `#[index]`:

```rust
pub struct User {
    pub username: String,  // REMOVE #[index] if it was there
}
```

### Step 2: Update database and generate

```bash
cargo run -p example-migration-workflow --features postgresql

cargo run -p toasty-cli --features postgresql -- migrate:generate \
  --message "remove username index" \
  --url "$DATABASE_URL" \
  --dir examples/migration-workflow/migration \
  --entity-dir examples/migration-workflow/entity
```

**Expected:**
```
✅ Detected 1 schema change(s):
   ✅ DropIndex { table: "users", index_name: "index_users_by_username" }
```

---

## Test Scenario 7: MODIFY COLUMN (Type Change)

### Step 1: Change column type

Edit `entity/src/lib.rs`:

```rust
pub struct User {
    pub name: String,  // Change from String to Option<String>
    // TO:
    pub name: Option<String>,
}
```

### Step 2: Update database and generate

```bash
cargo run -p example-migration-workflow --features postgresql

cargo run -p toasty-cli --features postgresql -- migrate:generate \
  --message "make name optional" \
  --url "$DATABASE_URL" \
  --dir examples/migration-workflow/migration \
  --entity-dir examples/migration-workflow/entity
```

**Expected:**
```
✅ Detected 1 schema change(s):
   ⚠️  ModifyColumn { table: "users", old: ColumnSnapshot { nullable: false }, new: ColumnSnapshot { nullable: true } }
```

---

## Test Scenario 8: MULTIPLE CHANGES

### Step 1: Make multiple changes

1. Add field to User: `pub age: Option<i32>`
2. Remove field from Post: remove `content`
3. Add new table: `Category`
4. Add index to Role: `#[index]` on description

### Step 2: Update and generate

```bash
cargo run -p example-migration-workflow --features postgresql

cargo run -p toasty-cli --features postgresql -- migrate:generate \
  --message "multiple schema changes" \
  --url "$DATABASE_URL" \
  --dir examples/migration-workflow/migration \
  --entity-dir examples/migration-workflow/entity
```

**Expected:**
```
✅ Detected 5 schema change(s):
   ✅ AddColumn { table: "users", column: "age" }
   ⚠️  DropColumn { table: "posts", column: "content" }
   ✅ CreateTable(Category)
   ✅ CreateIndex { table: "roles", index: "index_roles_by_description" }
```

---

## Complete Test Script

```bash
#!/bin/bash
export DATABASE_URL="postgresql://postgres:postgres@localhost:5433/postgres"

echo "=== Migration Testing Script ==="

# Test 1: Baseline
echo -e "\n1. Generate baseline migration..."
rm -rf examples/migration-workflow/migration
cargo run -p example-migration-workflow --features postgresql
cargo run -p toasty-cli --features postgresql -- migrate:generate \
  --message "baseline" \
  --url "$DATABASE_URL" \
  --dir examples/migration-workflow/migration \
  --entity-dir examples/migration-workflow/entity

# Test 2: Add column
echo -e "\n2. Testing ADD COLUMN..."
# (Manually edit entity/src/lib.rs to add bio field)
# cargo run -p example-migration-workflow --features postgresql
# cargo run -p toasty-cli --features postgresql -- migrate:generate --message "add bio" --url "$DATABASE_URL" --dir examples/migration-workflow/migration --entity-dir examples/migration-workflow/entity

# Test 3: Drop column
echo -e "\n3. Testing DROP COLUMN..."
# (Manually edit entity/src/lib.rs to remove description)
# cargo run -p example-migration-workflow --features postgresql
# cargo run -p toasty-cli --features postgresql -- migrate:generate --message "drop description" --url "$DATABASE_URL" --dir examples/migration-workflow/migration --entity-dir examples/migration-workflow/entity

echo -e "\n✅ Tests complete! Check migration files in examples/migration-workflow/migration/"
```

---

## Verification Checklist

After each test, verify:

- [ ] Migration file created with timestamp
- [ ] `up()` function has appropriate DDL
- [ ] `down()` function has rollback logic
- [ ] `.schema.json` updated with new state
- [ ] Change type correctly classified (additive vs destructive)

---

## Current Limitations

**Working:**
- ✅ CREATE TABLE
- ✅ DROP TABLE
- ✅ ADD COLUMN
- ✅ DROP COLUMN
- ✅ CREATE INDEX
- ✅ DROP INDEX

**Needs Testing:**
- ⏳ MODIFY COLUMN (type changes)
- ⏳ Column constraint changes
- ⏳ Multiple simultaneous changes

---

## Quick Test Commands

```bash
# Test ADD COLUMN
echo 'Add bio: Option<String> to User model, then:'
cargo run -p example-migration-workflow --features postgresql
cargo run -p toasty-cli --features postgresql -- migrate:generate --message "add bio" --url "$DATABASE_URL" --dir examples/migration-workflow/migration --entity-dir examples/migration-workflow/entity

# Test DROP COLUMN
echo 'Remove description from Role model, then:'
cargo run -p example-migration-workflow --features postgresql
cargo run -p toasty-cli --features postgresql -- migrate:generate --message "drop description" --url "$DATABASE_URL" --dir examples/migration-workflow/migration --entity-dir examples/migration-workflow/entity

# Test DROP TABLE
echo 'Comment out Post model, then:'
cargo run -p example-migration-workflow --features postgresql
cargo run -p toasty-cli --features postgresql -- migrate:generate --message "drop posts" --url "$DATABASE_URL" --dir examples/migration-workflow/migration --entity-dir examples/migration-workflow/entity

# Test ADD TABLE
echo 'Add Comment model, then:'
cargo run -p example-migration-workflow --features postgresql
cargo run -p toasty-cli --features postgresql -- migrate:generate --message "add comments" --url "$DATABASE_URL" --dir examples/migration-workflow/migration --entity-dir examples/migration-workflow/entity
```

---

## Expected Migration Examples

### ADD COLUMN Migration
```rust
fn up(&self, db: &mut dyn MigrationContext) -> Result<()> {
    db.add_column("users", ColumnDef {
        name: "bio".into(),
        ty: "text".into(),
        nullable: true,
        default: None
    })?;
    Ok(())
}
```

### DROP COLUMN Migration
```rust
fn up(&self, db: &mut dyn MigrationContext) -> Result<()> {
    db.drop_column("roles", "description")?;
    Ok(())
}

fn down(&self, db: &mut dyn MigrationContext) -> Result<()> {
    // Cannot automatically restore dropped column: roles.description
    Ok(())
}
```

### DROP TABLE Migration
```rust
fn up(&self, db: &mut dyn MigrationContext) -> Result<()> {
    db.drop_table("posts")?;
    Ok(())
}

fn down(&self, db: &mut dyn MigrationContext) -> Result<()> {
    // Cannot automatically recreate dropped table: posts
    // Manual intervention required
    Ok(())
}
```

---

## Tips

1. **Always run the app** after modifying entities to update the database
2. **The migration generator compares** database state vs previous snapshot
3. **Destructive changes** (DROP) show ⚠️ warning
4. **Review migrations** before applying to production
5. **Keep .schema.json** in version control to track schema history

This allows you to test all migration scenarios!
