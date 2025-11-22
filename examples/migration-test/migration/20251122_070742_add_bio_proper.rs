use toasty_migrate::{Migration, MigrationContext, ColumnDef, IndexDef};
use anyhow::Result;

pub struct Migration_20251122_070742_add_bio_proper;

impl Migration for Migration_20251122_070742_add_bio_proper {
    fn version(&self) -> &str {
        "20251122_070742_add_bio_proper"
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
