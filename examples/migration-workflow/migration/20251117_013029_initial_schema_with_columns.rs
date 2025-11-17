use toasty_migrate::{Migration, MigrationContext, ColumnDef, IndexDef};
use anyhow::Result;

pub struct Migration_20251117_013029_initial_schema_with_columns;

impl Migration for Migration_20251117_013029_initial_schema_with_columns {
    fn version(&self) -> &str {
        "20251117_013029_initial_schema_with_columns"
    }

    fn up(&self, db: &mut dyn MigrationContext) -> Result<()> {
        db.create_table("roles", vec![
            ColumnDef { name: "id".into(), ty: "text".into(), nullable: false, default: Some("''".into()) },
            ColumnDef { name: "name".into(), ty: "text".into(), nullable: true, default: None },
            ColumnDef { name: "description".into(), ty: "text".into(), nullable: true, default: None }
        ])?;
        db.create_table("users", vec![
            ColumnDef { name: "id".into(), ty: "text".into(), nullable: false, default: Some("''".into()) },
            ColumnDef { name: "name".into(), ty: "text".into(), nullable: true, default: None },
            ColumnDef { name: "username".into(), ty: "text".into(), nullable: true, default: None },
            ColumnDef { name: "email".into(), ty: "text".into(), nullable: true, default: None }
        ])?;
        db.create_table("posts", vec![
            ColumnDef { name: "id".into(), ty: "text".into(), nullable: false, default: Some("''".into()) },
            ColumnDef { name: "user_id".into(), ty: "text".into(), nullable: true, default: None },
            ColumnDef { name: "title".into(), ty: "text".into(), nullable: true, default: None },
            ColumnDef { name: "content".into(), ty: "text".into(), nullable: true, default: None }
        ])?;
        db.create_table("user_roles", vec![
            ColumnDef { name: "id".into(), ty: "text".into(), nullable: false, default: Some("''".into()) },
            ColumnDef { name: "user_id".into(), ty: "text".into(), nullable: true, default: None },
            ColumnDef { name: "role_id".into(), ty: "text".into(), nullable: true, default: None }
        ])?;
        Ok(())
    }

    fn down(&self, db: &mut dyn MigrationContext) -> Result<()> {
        db.drop_table("user_roles")?;
        db.drop_table("posts")?;
        db.drop_table("users")?;
        db.drop_table("roles")?;
        Ok(())
    }
}
