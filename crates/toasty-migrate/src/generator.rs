use crate::diff::{SchemaChange, SchemaDiff};
use anyhow::Result;

pub struct MigrationGenerator {
    migration_dir: std::path::PathBuf,
}

impl MigrationGenerator {
    pub fn new(migration_dir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            migration_dir: migration_dir.into(),
        }
    }

    pub fn generate(&self, diff: &SchemaDiff, description: &str) -> Result<MigrationFile> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let version = format!("{}_{}", timestamp, description.replace(' ', "_"));
        let filename = format!("{}.rs", version);

        let migration = MigrationFile {
            version: version.clone(),
            filename,
            up_statements: self.generate_up_statements(&diff.changes)?,
            down_statements: self.generate_down_statements(&diff.changes)?,
        };

        Ok(migration)
    }

    fn generate_up_statements(&self, changes: &[SchemaChange]) -> Result<Vec<String>> {
        let mut statements = Vec::new();

        for change in changes {
            match change {
                SchemaChange::CreateTable(table) => {
                    statements.push(format!(
                        "// Create table: {}",
                        table.name
                    ));
                    statements.push(format!(
                        "db.create_table(\"{}\", vec![/* columns */])?;",
                        table.name
                    ));
                }
                SchemaChange::DropTable(name) => {
                    statements.push(format!("db.drop_table(\"{}\")?;", name));
                }
                SchemaChange::AddColumn { table, column } => {
                    statements.push(format!(
                        "db.add_column(\"{}\", ColumnDef {{ name: \"{}\".into(), ty: \"{}\".into(), nullable: {} }})?;",
                        table, column.name, column.ty, column.nullable
                    ));
                }
                SchemaChange::DropColumn { table, column } => {
                    statements.push(format!("db.drop_column(\"{}\", \"{}\")?;", table, column));
                }
                SchemaChange::ModifyColumn { table, old, new } => {
                    statements.push(format!(
                        "// Modify column {}.{}: {} -> {}",
                        table, old.name, old.ty, new.ty
                    ));
                    statements.push(format!(
                        "// TODO: Implement column modification with data conversion"
                    ));
                }
                SchemaChange::CreateIndex { table, index } => {
                    statements.push(format!(
                        "db.create_index(\"{}\", IndexDef {{ name: \"{}\".into(), columns: vec![/* ... */], unique: {} }})?;",
                        table, index.name, index.unique
                    ));
                }
                SchemaChange::DropIndex { table, index_name } => {
                    statements.push(format!("db.drop_index(\"{}\", \"{}\")?;", table, index_name));
                }
            }
        }

        Ok(statements)
    }

    fn generate_down_statements(&self, changes: &[SchemaChange]) -> Result<Vec<String>> {
        let mut statements = Vec::new();

        // Reverse the changes
        for change in changes.iter().rev() {
            match change {
                SchemaChange::CreateTable(table) => {
                    statements.push(format!("db.drop_table(\"{}\")?;", table.name));
                }
                SchemaChange::DropTable(name) => {
                    statements.push(format!("// Cannot automatically recreate dropped table: {}", name));
                    statements.push(format!("// Manual intervention required"));
                }
                SchemaChange::AddColumn { table, column } => {
                    statements.push(format!("db.drop_column(\"{}\", \"{}\")?;", table, column.name));
                }
                SchemaChange::DropColumn { table, column } => {
                    statements.push(format!("// Cannot automatically restore dropped column: {}.{}", table, column));
                }
                SchemaChange::ModifyColumn { table, old, new: _ } => {
                    statements.push(format!("// Restore column {}.{} to original type", table, old.name));
                }
                SchemaChange::CreateIndex { table, index } => {
                    statements.push(format!("db.drop_index(\"{}\", \"{}\")?;", table, index.name));
                }
                SchemaChange::DropIndex { table, index_name } => {
                    statements.push(format!("// Recreate dropped index: {}.{}", table, index_name));
                }
            }
        }

        Ok(statements)
    }

    pub fn write_migration_file(&self, migration: &MigrationFile) -> Result<()> {
        std::fs::create_dir_all(&self.migration_dir)?;

        let file_path = self.migration_dir.join(&migration.filename);
        let content = self.generate_migration_code(migration)?;

        std::fs::write(file_path, content)?;
        Ok(())
    }

    fn generate_migration_code(&self, migration: &MigrationFile) -> Result<String> {
        let struct_name = migration
            .version
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>();

        let up_code = migration.up_statements.join("\n        ");
        let down_code = migration.down_statements.join("\n        ");

        Ok(format!(
            r#"use toasty_migrate::{{Migration, MigrationContext, ColumnDef, IndexDef}};
use anyhow::Result;

pub struct Migration_{};

impl Migration for Migration_{} {{
    fn version(&self) -> &str {{
        "{}"
    }}

    fn up(&self, db: &mut dyn MigrationContext) -> Result<()> {{
        {}
        Ok(())
    }}

    fn down(&self, db: &mut dyn MigrationContext) -> Result<()> {{
        {}
        Ok(())
    }}
}}
"#,
            struct_name, struct_name, migration.version, up_code, down_code
        ))
    }
}

#[derive(Debug, Clone)]
pub struct MigrationFile {
    pub version: String,
    pub filename: String,
    pub up_statements: Vec<String>,
    pub down_statements: Vec<String>,
}

pub trait Migration: Send + Sync {
    fn version(&self) -> &str;
    fn up(&self, db: &mut dyn crate::MigrationContext) -> Result<()>;
    fn down(&self, db: &mut dyn crate::MigrationContext) -> Result<()>;
}
