use toasty_migrate::{Migration, MigrationContext, ColumnDef, IndexDef};
use anyhow::Result;

pub struct Migration_20251122_073034_add_bio;

impl Migration for Migration_20251122_073034_add_bio {
    fn version(&self) -> &str {
        "20251122_073034_add_bio"
    }

    fn up(&self, db: &mut dyn MigrationContext) -> Result<()> {
        db.add_column("users", ColumnDef { name: "bio".into(), ty: "text".into(), nullable: true })?;
        // Modify column users.id: TEXT -> text
        // TODO: Implement column modification with data conversion
        // Modify column users.name: TEXT -> text
        // TODO: Implement column modification with data conversion
        // Modify column users.email: TEXT -> text
        // TODO: Implement column modification with data conversion
        db.create_index("users", IndexDef { name: "index_users_by_email".into(), columns: vec!["email".into()], unique: true })?;
        Ok(())
    }

    fn down(&self, db: &mut dyn MigrationContext) -> Result<()> {
        db.drop_index("users", "index_users_by_email")?;
        // Restore column users.email to original type
        // Restore column users.name to original type
        // Restore column users.id to original type
        db.drop_column("users", "bio")?;
        Ok(())
    }
}
