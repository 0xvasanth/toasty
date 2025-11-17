use toasty_migrate::{Migration, MigrationContext, ColumnDef, IndexDef};
use anyhow::Result;

pub struct Migration_20251117_012700_initial_schema;

impl Migration for Migration_20251117_012700_initial_schema {
    fn version(&self) -> &str {
        "20251117_012700_initial_schema"
    }

    fn up(&self, db: &mut dyn MigrationContext) -> Result<()> {
        
        Ok(())
    }

    fn down(&self, db: &mut dyn MigrationContext) -> Result<()> {
        
        Ok(())
    }
}
