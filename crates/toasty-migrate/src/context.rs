use crate::{ColumnDef, IndexDef, MigrationContext};
use anyhow::Result;

/// SQL-based migration context for SQLite, PostgreSQL, MySQL
pub struct SqlMigrationContext {
    statements: Vec<String>,
    flavor: SqlFlavor,
}

#[derive(Debug, Clone, Copy)]
pub enum SqlFlavor {
    Sqlite,
    PostgreSQL,
    MySQL,
}

impl SqlMigrationContext {
    pub fn new(flavor: SqlFlavor) -> Self {
        Self {
            statements: Vec::new(),
            flavor,
        }
    }

    pub fn statements(&self) -> &[String] {
        &self.statements
    }

    fn add_statement(&mut self, sql: String) {
        self.statements.push(sql);
    }
}

impl MigrationContext for SqlMigrationContext {
    fn execute_sql(&mut self, sql: &str) -> Result<()> {
        self.add_statement(sql.to_string());
        Ok(())
    }

    fn create_table(&mut self, name: &str, columns: Vec<ColumnDef>) -> Result<()> {
        let column_defs: Vec<String> = columns
            .iter()
            .map(|col| {
                let mut def = format!("{} {}", col.name, col.ty);
                if !col.nullable {
                    def.push_str(" NOT NULL");
                }
                if let Some(default) = &col.default {
                    def.push_str(&format!(" DEFAULT {}", default));
                }
                def
            })
            .collect();

        let sql = format!(
            "CREATE TABLE {} (\n  {}\n);",
            name,
            column_defs.join(",\n  ")
        );

        self.add_statement(sql);
        Ok(())
    }

    fn drop_table(&mut self, name: &str) -> Result<()> {
        self.add_statement(format!("DROP TABLE {};", name));
        Ok(())
    }

    fn add_column(&mut self, table: &str, column: ColumnDef) -> Result<()> {
        let mut def = format!("{} {}", column.name, column.ty);
        if !column.nullable {
            def.push_str(" NOT NULL");
        }
        if let Some(default) = &column.default {
            def.push_str(&format!(" DEFAULT {}", default));
        }

        let sql = match self.flavor {
            SqlFlavor::Sqlite => {
                // SQLite has limited ALTER TABLE support
                format!("ALTER TABLE {} ADD COLUMN {};", table, def)
            }
            SqlFlavor::PostgreSQL | SqlFlavor::MySQL => {
                format!("ALTER TABLE {} ADD COLUMN {};", table, def)
            }
        };

        self.add_statement(sql);
        Ok(())
    }

    fn drop_column(&mut self, table: &str, column: &str) -> Result<()> {
        let sql = match self.flavor {
            SqlFlavor::Sqlite => {
                // SQLite doesn't support DROP COLUMN directly
                // Need to recreate table
                format!("-- SQLite: Cannot DROP COLUMN {}. Requires table recreation", column)
            }
            SqlFlavor::PostgreSQL | SqlFlavor::MySQL => {
                format!("ALTER TABLE {} DROP COLUMN {};", table, column)
            }
        };

        self.add_statement(sql);
        Ok(())
    }

    fn create_index(&mut self, table: &str, index: IndexDef) -> Result<()> {
        let unique = if index.unique { "UNIQUE " } else { "" };
        let columns = index.columns.join(", ");

        let sql = format!(
            "CREATE {}INDEX {} ON {} ({});",
            unique, index.name, table, columns
        );

        self.add_statement(sql);
        Ok(())
    }

    fn drop_index(&mut self, _table: &str, index_name: &str) -> Result<()> {
        let sql = match self.flavor {
            SqlFlavor::Sqlite | SqlFlavor::PostgreSQL => {
                format!("DROP INDEX {};", index_name)
            }
            SqlFlavor::MySQL => {
                // MySQL requires table name
                format!("DROP INDEX {} ON {};", index_name, _table)
            }
        };

        self.add_statement(sql);
        Ok(())
    }
}

/// NoSQL-based migration context for MongoDB, DynamoDB
pub struct NoSqlMigrationContext {
    operations: Vec<NoSqlOperation>,
}

#[derive(Debug, Clone)]
pub enum NoSqlOperation {
    CreateCollection { name: String },
    DropCollection { name: String },
    CreateIndex { collection: String, index: IndexDef },
    DropIndex { collection: String, index_name: String },
    // NoSQL doesn't need ADD/DROP column - documents are flexible
}

impl NoSqlMigrationContext {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    pub fn operations(&self) -> &[NoSqlOperation] {
        &self.operations
    }
}

impl MigrationContext for NoSqlMigrationContext {
    fn execute_sql(&mut self, _sql: &str) -> Result<()> {
        Err(anyhow::anyhow!("SQL execution not supported in NoSQL context"))
    }

    fn create_table(&mut self, name: &str, _columns: Vec<ColumnDef>) -> Result<()> {
        self.operations.push(NoSqlOperation::CreateCollection {
            name: name.to_string(),
        });
        Ok(())
    }

    fn drop_table(&mut self, name: &str) -> Result<()> {
        self.operations.push(NoSqlOperation::DropCollection {
            name: name.to_string(),
        });
        Ok(())
    }

    fn add_column(&mut self, _table: &str, _column: ColumnDef) -> Result<()> {
        // NoSQL databases don't require schema changes for adding fields
        // Documents can have different fields
        Ok(())
    }

    fn drop_column(&mut self, _table: &str, _column: &str) -> Result<()> {
        // NoSQL databases don't require schema changes for removing fields
        Ok(())
    }

    fn create_index(&mut self, table: &str, index: IndexDef) -> Result<()> {
        self.operations.push(NoSqlOperation::CreateIndex {
            collection: table.to_string(),
            index,
        });
        Ok(())
    }

    fn drop_index(&mut self, table: &str, index_name: &str) -> Result<()> {
        self.operations.push(NoSqlOperation::DropIndex {
            collection: table.to_string(),
            index_name: index_name.to_string(),
        });
        Ok(())
    }
}
