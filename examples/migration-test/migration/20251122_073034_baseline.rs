use toasty_migrate::{Migration, MigrationContext, ColumnDef, IndexDef};
use anyhow::Result;

pub struct Migration_20251122_073034_baseline;

impl Migration for Migration_20251122_073034_baseline {
    fn version(&self) -> &str {
        "20251122_073034_baseline"
    }

    fn up(&self, db: &mut dyn MigrationContext) -> Result<()> {
        db.create_table("users", vec![
            ColumnDef { name: "id".into(), ty: "text".into(), nullable: false, default: Some("''".into()) },
            ColumnDef { name: "name".into(), ty: "text".into(), nullable: false, default: Some("''".into()) },
            ColumnDef { name: "email".into(), ty: "text".into(), nullable: false, default: Some("''".into()) }
        ])?;
        db.create_index("users", IndexDef { name: "index_users_by_email".into(), columns: vec!["email".into()], unique: true })?;
        Ok(())
    }

    fn down(&self, db: &mut dyn MigrationContext) -> Result<()> {
        db.drop_table("users")?;
        Ok(())
    }
}
