use toasty_migrate::{Migration, MigrationContext, ColumnDef, IndexDef};
use anyhow::Result;

pub struct Migration_20251117_014221_initial_schema;

impl Migration for Migration_20251117_014221_initial_schema {
    fn version(&self) -> &str {
        "20251117_014221_initial_schema"
    }

    fn up(&self, db: &mut dyn MigrationContext) -> Result<()> {
        db.create_table("users", vec![
            ColumnDef { name: "id".into(), ty: "text".into(), nullable: false, default: Some("''".into()) },
            ColumnDef { name: "name".into(), ty: "text".into(), nullable: true, default: None },
            ColumnDef { name: "username".into(), ty: "text".into(), nullable: true, default: None },
            ColumnDef { name: "email".into(), ty: "text".into(), nullable: true, default: None }
        ])?;
        db.create_index("users", IndexDef { name: "index_users_by_email".into(), columns: vec!["email".into()], unique: true })?;
        db.create_index("users", IndexDef { name: "index_users_by_username".into(), columns: vec!["username".into()], unique: true })?;
        db.create_table("posts", vec![
            ColumnDef { name: "id".into(), ty: "text".into(), nullable: false, default: Some("''".into()) },
            ColumnDef { name: "user_id".into(), ty: "text".into(), nullable: true, default: None },
            ColumnDef { name: "title".into(), ty: "text".into(), nullable: true, default: None },
            ColumnDef { name: "content".into(), ty: "text".into(), nullable: true, default: None }
        ])?;
        db.create_index("posts", IndexDef { name: "index_posts_by_user_id".into(), columns: vec!["user_id".into()], unique: false })?;
        db.create_table("roles", vec![
            ColumnDef { name: "id".into(), ty: "text".into(), nullable: false, default: Some("''".into()) },
            ColumnDef { name: "name".into(), ty: "text".into(), nullable: true, default: None },
            ColumnDef { name: "description".into(), ty: "text".into(), nullable: true, default: None }
        ])?;
        db.create_index("roles", IndexDef { name: "index_roles_by_name".into(), columns: vec!["name".into()], unique: true })?;
        db.create_table("user_roles", vec![
            ColumnDef { name: "id".into(), ty: "text".into(), nullable: false, default: Some("''".into()) },
            ColumnDef { name: "user_id".into(), ty: "text".into(), nullable: true, default: None },
            ColumnDef { name: "role_id".into(), ty: "text".into(), nullable: true, default: None }
        ])?;
        db.create_index("user_roles", IndexDef { name: "index_user_roles_by_role_id".into(), columns: vec!["role_id".into()], unique: false })?;
        db.create_index("user_roles", IndexDef { name: "index_user_roles_by_user_id".into(), columns: vec!["user_id".into()], unique: false })?;
        Ok(())
    }

    fn down(&self, db: &mut dyn MigrationContext) -> Result<()> {
        db.drop_table("user_roles")?;
        db.drop_table("roles")?;
        db.drop_table("posts")?;
        db.drop_table("users")?;
        Ok(())
    }
}
