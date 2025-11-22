use toasty_migrate::{Migration, MigrationContext, ColumnDef, IndexDef};
use anyhow::Result;

pub struct Migration_20251122_063248_add_user_email;

impl Migration for Migration_20251122_063248_add_user_email {
    fn version(&self) -> &str {
        "20251122_063248_add_user_email"
    }

    fn up(&self, db: &mut dyn MigrationContext) -> Result<()> {
        db.add_column("users", ColumnDef { name: "email".into(), ty: "String".into(), nullable: false })?;
        db.create_index("users", IndexDef { name: "idx_users_email".into(), columns: vec![/* ... */], unique: true })?;
        Ok(())
    }

    fn down(&self, db: &mut dyn MigrationContext) -> Result<()> {
        db.drop_index("users", "idx_users_email")?;
        db.drop_column("users", "email")?;
        Ok(())
    }
}
