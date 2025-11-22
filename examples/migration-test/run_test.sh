#!/bin/bash
set -e

echo "=== Migration Incremental Test ==="
echo ""

# Setup
export DATABASE_URL="sqlite:examples/migration-test/test.db"
rm -f examples/migration-test/test.db
rm -rf examples/migration-test/migration

echo "Step 1: Generate baseline migration"
echo "------------------------------------"
cargo run -p toasty-cli -- migrate:generate \
  --message "baseline schema" \
  --url "$DATABASE_URL" \
  --dir examples/migration-test/migration \
  --entity-dir examples/migration-test/entity

echo ""
echo "📄 Migration 1 contents:"
cat examples/migration-test/migration/*.rs | grep -A2 "fn up"
echo ""

# Modify entity - add bio field
echo "Step 2: Add bio field to User"
echo "------------------------------"
sed -i.bak 's|// Step 2: Uncomment to add bio field|pub bio: Option<String>,|' examples/migration-test/entity/src/lib.rs

cargo run -p toasty-cli -- migrate:generate \
  --message "add bio field" \
  --url "$DATABASE_URL" \
  --dir examples/migration-test/migration \
  --entity-dir examples/migration-test/entity

echo ""
echo "📄 Migration 2 contents:"
ls -lt examples/migration-test/migration/*.rs | head -2 | tail -1 | awk '{print $NF}' | xargs cat | grep -A5 "fn up"
echo ""

echo "✅ Test Complete!"
echo ""
echo "Verification:"
echo "  - Migration 1: Should have CREATE TABLE"
echo "  - Migration 2: Should have ONLY ADD COLUMN (not CREATE TABLE)"

