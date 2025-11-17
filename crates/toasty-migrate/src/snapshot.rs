use toasty_core::schema::db::Schema;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaSnapshot {
    pub version: String,
    pub timestamp: String,
    pub tables: Vec<TableSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSnapshot {
    pub name: String,
    pub columns: Vec<ColumnSnapshot>,
    pub indices: Vec<IndexSnapshot>,
    pub primary_key: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnSnapshot {
    pub name: String,
    pub ty: String,
    pub nullable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexSnapshot {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
    pub primary_key: bool,
}

impl SchemaSnapshot {
    pub fn from_schema(schema: &Schema) -> Self {
        let timestamp = chrono::Utc::now().to_rfc3339();
        let mut tables = Vec::new();

        for table in &schema.tables {
            let mut columns = Vec::new();
            for column in &table.columns {
                columns.push(ColumnSnapshot {
                    name: column.name.clone(),
                    ty: format!("{:?}", column.ty),
                    nullable: column.nullable,
                });
            }

            let mut indices = Vec::new();
            for index in &table.indices {
                let column_names: Vec<String> = index
                    .columns
                    .iter()
                    .map(|col_ref| table.column(col_ref.column).name.clone())
                    .collect();

                indices.push(IndexSnapshot {
                    name: index.name.clone(),
                    columns: column_names,
                    unique: index.unique,
                    primary_key: index.primary_key,
                });
            }

            let primary_key: Vec<String> = table
                .primary_key_columns()
                .map(|col| col.name.clone())
                .collect();

            tables.push(TableSnapshot {
                name: table.name.clone(),
                columns,
                indices,
                primary_key,
            });
        }

        SchemaSnapshot {
            version: "1.0".to_string(),
            timestamp,
            tables,
        }
    }
}

pub fn save_snapshot(snapshot: &SchemaSnapshot, path: impl AsRef<Path>) -> Result<()> {
    let json = serde_json::to_string_pretty(snapshot)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn load_snapshot(path: impl AsRef<Path>) -> Result<SchemaSnapshot> {
    let json = std::fs::read_to_string(path)?;
    let snapshot = serde_json::from_str(&json)?;
    Ok(snapshot)
}
