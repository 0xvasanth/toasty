pub mod snapshot;
pub mod diff;
pub mod generator;
pub mod tracker;
pub mod runner;
pub mod loader;
pub mod context;
pub mod introspect;
pub mod parser;

pub use snapshot::{SchemaSnapshot, save_snapshot, load_snapshot};
pub use diff::{SchemaChange, SchemaDiff, detect_changes};
pub use generator::{Migration, MigrationGenerator, MigrationFile};
pub use tracker::MigrationTracker;
pub use runner::{MigrationRunner, MigrationStatus};
pub use loader::{MigrationLoader, MigrationFileInfo};
pub use context::{SqlMigrationContext, NoSqlMigrationContext, SqlFlavor, NoSqlOperation};
pub use introspect::{SchemaIntrospector, SqlIntrospector, MongoDbIntrospector};
pub use parser::EntityParser;

use anyhow::Result;

/// Migration context provides database operations for migrations
pub trait MigrationContext {
    /// Execute a raw SQL statement (SQL databases only)
    fn execute_sql(&mut self, sql: &str) -> Result<()>;

    /// Create a table
    fn create_table(&mut self, name: &str, columns: Vec<ColumnDef>) -> Result<()>;

    /// Drop a table
    fn drop_table(&mut self, name: &str) -> Result<()>;

    /// Add a column to a table
    fn add_column(&mut self, table: &str, column: ColumnDef) -> Result<()>;

    /// Drop a column from a table
    fn drop_column(&mut self, table: &str, column: &str) -> Result<()>;

    /// Create an index
    fn create_index(&mut self, table: &str, index: IndexDef) -> Result<()>;

    /// Drop an index
    fn drop_index(&mut self, table: &str, index_name: &str) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: String,
    pub ty: String,
    pub nullable: bool,
    pub default: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IndexDef {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
}
