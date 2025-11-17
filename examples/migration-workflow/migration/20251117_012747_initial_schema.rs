use toasty_migrate::{Migration, MigrationContext, ColumnDef, IndexDef};
use anyhow::Result;

pub struct Migration_20251117_012747_initial_schema;

impl Migration for Migration_20251117_012747_initial_schema {
    fn version(&self) -> &str {
        "20251117_012747_initial_schema"
    }

    fn up(&self, db: &mut dyn MigrationContext) -> Result<()> {
        // Create table: posts
        db.create_table("posts", vec![/* columns */])?;
        // Create table: roles
        db.create_table("roles", vec![/* columns */])?;
        // Create table: users
        db.create_table("users", vec![/* columns */])?;
        // Create table: user_roles
        db.create_table("user_roles", vec![/* columns */])?;
        Ok(())
    }

    fn down(&self, db: &mut dyn MigrationContext) -> Result<()> {
        db.drop_table("user_roles")?;
        db.drop_table("users")?;
        db.drop_table("roles")?;
        db.drop_table("posts")?;
        Ok(())
    }
}
