use toasty_migrate::{Migration, MigrationContext, ColumnDef, IndexDef};
use anyhow::Result;

pub struct Migration_20251122_074505_add_bio;

impl Migration for Migration_20251122_074505_add_bio {
    fn version(&self) -> &str {
        "20251122_074505_add_bio"
    }

    fn up(&self, db: &mut dyn MigrationContext) -> Result<()> {
        db.add_column("users", ColumnDef { name: "bio".into(), ty: "text".into(), nullable: true })?;
        Ok(())
    }

    fn down(&self, db: &mut dyn MigrationContext) -> Result<()> {
        db.drop_column("users", "bio")?;
        Ok(())
    }
}
