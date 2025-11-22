# Migration Incremental Generation Issue & Solution

## The Problem

**Current Behavior:**
- Migration 1: CREATE TABLE users (id, name, email)
- Add bio field to entity
- Migration 2: CREATE TABLE users (id, name, email, bio) ❌ WRONG!

**Expected:**
- Migration 2: ADD COLUMN bio ✅

## Root Cause

The system compares DATABASE (current) vs ENTITIES (desired).
But if migrations haven't been applied, database is empty, so it regenerates everything.

## Attempted Solutions

### Attempt 1: Use .schema.json
- Compare .schema.json (old) vs entities (new)
- **Problem**: Fragile if .schema.json deleted

### Attempt 2: Apply migrations before comparing
- Apply existing migrations to DB first
- Then compare DB vs entities
- **Problem**: Complex execution, SQLite introspection issues

### Attempt 3: Prisma's Shadow Database ✅

**How Prisma Does It:**
1. Create temporary "shadow" database
2. Apply ALL existing migrations to shadow DB
3. Shadow DB = "state after all migrations"
4. Compare: Shadow DB vs Schema file
5. Generate: ONLY the diff
6. Delete shadow DB

**Benefits:**
- No .schema.json dependency
- Migrations are source of truth
- Always incremental
- Works offline (SQLite shadow)

## Implementation Plan

### Shadow Database Module
```rust
pub struct ShadowDatabase {
    temp_db: tempfile::NamedTempFile,
}

impl ShadowDatabase {
    fn new() -> Result<Self>;
    fn apply_migrations(&self, migration_dir: &Path) -> Result<SchemaSnapshot>;
    fn introspect(&self) -> Result<SchemaSnapshot>;
}
```

### Updated Workflow
```rust
// In migrate:generate
let shadow = ShadowDatabase::new()?;
let current = shadow.apply_migrations(&migration_dir)?; // State after migrations
let desired = parse_entities(&entity_dir)?; // What we want
let diff = detect_changes(&current, &desired)?; // Only new changes!
```

### Migration Execution
Since we can't execute .rs files, we use .schema.json as a cache:
- .schema.json = accumulated entity state after all migrations
- Each migration updates .schema.json
- Shadow DB uses .schema.json (not actual SQL execution)

## Status

- ✅ Shadow module created
- ✅ Uses .schema.json for state tracking
- ⏳ Testing in progress
- ⏳ Needs: SQLite introspection for shadow DB

## Test Plan

1. Generate baseline (no .schema.json) → CREATE TABLE
2. Saves entity schema to .schema.json
3. Add bio field
4. Shadow DB loads .schema.json (users without bio)
5. Compare: users(no bio) vs users(with bio)
6. Generate: ADD COLUMN bio ✅

This approach combines best of both:
- Migrations as source of truth
- .schema.json as optimization (can be regenerated)
- Prisma-style shadow database
