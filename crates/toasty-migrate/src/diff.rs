use crate::snapshot::{SchemaSnapshot, TableSnapshot, ColumnSnapshot, IndexSnapshot};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct SchemaDiff {
    pub changes: Vec<SchemaChange>,
}

#[derive(Debug, Clone)]
pub enum SchemaChange {
    // Table changes
    CreateTable(TableSnapshot),
    DropTable(String),

    // Column changes
    AddColumn { table: String, column: ColumnSnapshot },
    DropColumn { table: String, column: String },
    ModifyColumn { table: String, old: ColumnSnapshot, new: ColumnSnapshot },

    // Index changes
    CreateIndex { table: String, index: IndexSnapshot },
    DropIndex { table: String, index_name: String },
}

impl SchemaChange {
    pub fn is_destructive(&self) -> bool {
        matches!(
            self,
            SchemaChange::DropTable(_)
                | SchemaChange::DropColumn { .. }
                | SchemaChange::ModifyColumn { .. }
        )
    }

    pub fn is_additive(&self) -> bool {
        matches!(
            self,
            SchemaChange::CreateTable(_)
                | SchemaChange::AddColumn { .. }
                | SchemaChange::CreateIndex { .. }
        )
    }
}

pub fn detect_changes(old: &SchemaSnapshot, new: &SchemaSnapshot) -> Result<SchemaDiff> {
    let mut changes = Vec::new();

    // Build maps for quick lookup
    let old_tables: std::collections::HashMap<_, _> =
        old.tables.iter().map(|t| (&t.name, t)).collect();
    let new_tables: std::collections::HashMap<_, _> =
        new.tables.iter().map(|t| (&t.name, t)).collect();

    // Detect dropped tables
    for (table_name, _table) in &old_tables {
        if !new_tables.contains_key(table_name) {
            changes.push(SchemaChange::DropTable((*table_name).clone()));
        }
    }

    // Detect new tables
    for (table_name, table) in &new_tables {
        if !old_tables.contains_key(table_name) {
            changes.push(SchemaChange::CreateTable((*table).clone()));
        }
    }

    // Detect column and index changes within existing tables
    for (table_name, new_table) in &new_tables {
        if let Some(old_table) = old_tables.get(table_name) {
            detect_table_changes(&mut changes, table_name, old_table, new_table);
        }
    }

    Ok(SchemaDiff { changes })
}

fn detect_table_changes(
    changes: &mut Vec<SchemaChange>,
    table_name: &str,
    old_table: &TableSnapshot,
    new_table: &TableSnapshot,
) {
    // Build column maps
    let old_columns: std::collections::HashMap<_, _> =
        old_table.columns.iter().map(|c| (&c.name, c)).collect();
    let new_columns: std::collections::HashMap<_, _> =
        new_table.columns.iter().map(|c| (&c.name, c)).collect();

    // Detect dropped columns
    for (col_name, _col) in &old_columns {
        if !new_columns.contains_key(col_name) {
            changes.push(SchemaChange::DropColumn {
                table: table_name.to_string(),
                column: (*col_name).clone(),
            });
        }
    }

    // Detect new columns
    for (col_name, col) in &new_columns {
        if !old_columns.contains_key(col_name) {
            changes.push(SchemaChange::AddColumn {
                table: table_name.to_string(),
                column: (*col).clone(),
            });
        }
    }

    // Detect modified columns
    for (col_name, new_col) in &new_columns {
        if let Some(old_col) = old_columns.get(col_name) {
            if old_col.ty != new_col.ty || old_col.nullable != new_col.nullable {
                changes.push(SchemaChange::ModifyColumn {
                    table: table_name.to_string(),
                    old: (*old_col).clone(),
                    new: (*new_col).clone(),
                });
            }
        }
    }

    // Detect index changes
    let old_indices: std::collections::HashMap<_, _> =
        old_table.indices.iter().map(|i| (&i.name, i)).collect();
    let new_indices: std::collections::HashMap<_, _> =
        new_table.indices.iter().map(|i| (&i.name, i)).collect();

    // Dropped indices
    for (idx_name, _idx) in &old_indices {
        if !new_indices.contains_key(idx_name) {
            changes.push(SchemaChange::DropIndex {
                table: table_name.to_string(),
                index_name: (*idx_name).clone(),
            });
        }
    }

    // New indices
    for (idx_name, idx) in &new_indices {
        if !old_indices.contains_key(idx_name) {
            changes.push(SchemaChange::CreateIndex {
                table: table_name.to_string(),
                index: (*idx).clone(),
            });
        }
    }
}
