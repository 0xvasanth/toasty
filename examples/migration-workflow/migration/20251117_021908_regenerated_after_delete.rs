use toasty_migrate::{Migration, MigrationContext, ColumnDef, IndexDef};
use anyhow::Result;

pub struct Migration_20251117_021908_regenerated_after_delete;

impl Migration for Migration_20251117_021908_regenerated_after_delete {
    fn version(&self) -> &str {
        "20251117_021908_regenerated_after_delete"
    }

    fn up(&self, db: &mut dyn MigrationContext) -> Result<()> {
        db.drop_table("user_roles")?;
        db.create_table("userroles", vec![
            ColumnDef { name: "id".into(), ty: "text".into(), nullable: false, default: Some("''".into()) },
            ColumnDef { name: "user_id".into(), ty: "text".into(), nullable: false, default: Some("''".into()) },
            ColumnDef { name: "role_id".into(), ty: "text".into(), nullable: false, default: Some("''".into()) }
        ])?;
        db.create_index("userroles", IndexDef { name: "index_userroles_by_user_id".into(), columns: vec!["user_id".into()], unique: false })?;
        db.create_index("userroles", IndexDef { name: "index_userroles_by_role_id".into(), columns: vec!["role_id".into()], unique: false })?;
        db.create_table("dummys", vec![
            ColumnDef { name: "id".into(), ty: "text".into(), nullable: false, default: Some("''".into()) },
            ColumnDef { name: "user_id".into(), ty: "text".into(), nullable: false, default: Some("''".into()) },
            ColumnDef { name: "title".into(), ty: "text".into(), nullable: false, default: Some("''".into()) },
            ColumnDef { name: "content".into(), ty: "text".into(), nullable: false, default: Some("''".into()) }
        ])?;
        db.create_index("dummys", IndexDef { name: "index_dummys_by_user_id".into(), columns: vec!["user_id".into()], unique: false })?;
        // Modify column roles.name: text -> text
        // TODO: Implement column modification with data conversion
        db.drop_index("roles", "roles_pkey")?;
        // Modify column users.username: text -> text
        // TODO: Implement column modification with data conversion
        // Modify column users.email: text -> text
        // TODO: Implement column modification with data conversion
        // Modify column users.name: text -> text
        // TODO: Implement column modification with data conversion
        db.drop_index("users", "users_pkey")?;
        // Modify column posts.user_id: text -> text
        // TODO: Implement column modification with data conversion
        // Modify column posts.content: text -> text
        // TODO: Implement column modification with data conversion
        // Modify column posts.title: text -> text
        // TODO: Implement column modification with data conversion
        db.drop_index("posts", "posts_pkey")?;
        Ok(())
    }

    fn down(&self, db: &mut dyn MigrationContext) -> Result<()> {
        // Recreate dropped index: posts.posts_pkey
        // Restore column posts.title to original type
        // Restore column posts.content to original type
        // Restore column posts.user_id to original type
        // Recreate dropped index: users.users_pkey
        // Restore column users.name to original type
        // Restore column users.email to original type
        // Restore column users.username to original type
        // Recreate dropped index: roles.roles_pkey
        // Restore column roles.name to original type
        db.drop_table("dummys")?;
        db.drop_table("userroles")?;
        // Cannot automatically recreate dropped table: user_roles
        // Manual intervention required
        Ok(())
    }
}
