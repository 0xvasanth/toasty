use toasty_migrate::{Migration, MigrationContext, ColumnDef, IndexDef};
use anyhow::Result;

pub struct Migration_20251122_074804_test;

impl Migration for Migration_20251122_074804_test {
    fn version(&self) -> &str {
        "20251122_074804_test"
    }

    fn up(&self, db: &mut dyn MigrationContext) -> Result<()> {
        db.add_column("users", ColumnDef { name: "age".into(), ty: "text".into(), nullable: true })?;
        Ok(())
    }

    fn down(&self, db: &mut dyn MigrationContext) -> Result<()> {
        db.drop_column("users", "age")?;
        Ok(())
    }
}
