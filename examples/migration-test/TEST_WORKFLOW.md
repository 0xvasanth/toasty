# Migration Incremental Test Workflow

This example demonstrates that migrations are INCREMENTAL - only detecting new changes, not regenerating everything.

## Test Scenario

### Step 1: Initial Schema
- User model with: id, name, email
- Generate baseline migration

### Step 2: Add Field
- Add `bio` field to User
- Generate migration
- **Expected**: Migration with ONLY `ADD COLUMN bio` (not full CREATE TABLE)

### Step 3: Add Index
- Add `#[index]` to name field
- Generate migration  
- **Expected**: Migration with ONLY `CREATE INDEX` (not previous changes)

### Step 4: Verify
- Check that each migration contains ONLY its changes
- Migrations are cumulative but separate

## Running the Test

```bash
# Step 1: Baseline
cargo run --bin migration-test-step1

# Step 2: Add bio field
cargo run --bin migration-test-step2

# Step 3: Add index
cargo run --bin migration-test-step3

# Or run all steps
./run-migration-test.sh
```

## Expected Results

**Migration 1 (baseline):**
```rust
fn up() {
    db.create_table("users", vec![...]) // Full table
}
```

**Migration 2 (add bio):**
```rust
fn up() {
    db.add_column("users", ColumnDef { name: "bio", ... }) // ONLY new column
}
```

**Migration 3 (add index):**
```rust
fn up() {
    db.create_index("users", IndexDef { name: "idx_name", ... }) // ONLY new index
}
```

This proves migrations are INCREMENTAL! ✅
