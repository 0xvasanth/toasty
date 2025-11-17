use crate::snapshot::*;
use anyhow::Result;
use std::path::Path;

/// Parse Rust entity files to extract schema
pub struct EntityParser {
    entity_dir: std::path::PathBuf,
}

impl EntityParser {
    pub fn new(entity_dir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            entity_dir: entity_dir.into(),
        }
    }

    /// Parse entity files and build schema snapshot
    pub fn parse_entities(&self) -> Result<SchemaSnapshot> {
        println!("📖 Parsing entity files from: {}", self.entity_dir.display());

        // Read all .rs files in the entity directory
        let lib_path = self.entity_dir.join("src/lib.rs");

        if !lib_path.exists() {
            return Err(anyhow::anyhow!(
                "Entity lib.rs not found at: {}. Run 'toasty init' first.",
                lib_path.display()
            ));
        }

        let content = std::fs::read_to_string(&lib_path)?;
        let tables = self.parse_models_from_content(&content)?;

        println!("✅ Parsed {} model(s) from entity files", tables.len());

        Ok(SchemaSnapshot {
            version: "1.0".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            tables,
        })
    }

    fn parse_models_from_content(&self, content: &str) -> Result<Vec<TableSnapshot>> {
        let mut tables = Vec::new();

        // Simple regex-based parsing (for MVP - could use syn for full parsing)
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // Look for #[derive(Debug, toasty::Model)] or #[derive(toasty::Model)]
            if line.contains("#[derive") && line.contains("toasty::Model") {
                // Next lines should have the struct
                i += 1;
                while i < lines.len() {
                    let struct_line = lines[i].trim();
                    if struct_line.starts_with("pub struct ") {
                        if let Some(table) = self.parse_struct(&lines, i)? {
                            tables.push(table);
                        }
                        break;
                    }
                    i += 1;
                }
            }
            i += 1;
        }

        Ok(tables)
    }

    fn parse_struct(&self, lines: &[&str], start: usize) -> Result<Option<TableSnapshot>> {
        // Extract struct name
        let struct_line = lines[start].trim();
        let parts: Vec<&str> = struct_line.split_whitespace().collect();
        if parts.len() < 3 {
            return Ok(None);
        }

        let struct_name = parts[2].trim_end_matches(" {").to_string();
        let table_name = struct_name.to_lowercase() + "s"; // Simple pluralization

        let mut columns = Vec::new();
        let mut indices = Vec::new();
        let mut primary_key = Vec::new();
        let mut has_key = false;

        // Parse fields
        let mut i = start + 1;
        while i < lines.len() {
            let line = lines[i].trim();

            if line == "}" {
                break;
            }

            // Check for attributes in previous lines
            let mut is_key = false;
            let mut is_unique = false;
            let mut is_index = false;
            let mut is_relation = false;

            // Look back for attributes on previous line only
            if i > 0 {
                let prev_line = lines[i - 1].trim();
                if prev_line.contains("#[key]") {
                    is_key = true;
                }
                if prev_line.contains("#[unique]") {
                    is_unique = true;
                }
                if prev_line.contains("#[index]") {
                    is_index = true;
                }
                if prev_line.contains("#[has_many]") || prev_line.contains("#[belongs_to]") {
                    is_relation = true;
                }
            }

            // Parse field: pub name: Type,
            if line.starts_with("pub ") && line.contains(":") {
                // Quick check: skip if this is obviously a relationship type
                if line.contains("HasMany<") || line.contains("BelongsTo<") {
                    i += 1;
                    continue;
                }

                // Skip if previous line has relationship attribute
                if is_relation {
                    i += 1;
                    continue;
                }

                let field_parts: Vec<&str> = line.split(':').collect();
                if field_parts.len() >= 2 {
                    let field_name = field_parts[0]
                        .trim()
                        .trim_start_matches("pub ")
                        .to_string();

                    let field_type = field_parts[1]
                        .trim()
                        .trim_end_matches(',')
                        .to_string();

                    // Skip if this field name is a duplicate (already added as key)
                    let is_duplicate = columns.iter().any(|c: &ColumnSnapshot| c.name == field_name);
                    if is_duplicate {
                        i += 1;
                        continue;
                    }

                    // Determine nullable and type
                    let (nullable, clean_type) = if field_type.starts_with("Option<") {
                        let inner = field_type
                            .trim_start_matches("Option<")
                            .trim_end_matches('>')
                            .to_string();
                        (true, inner)
                    } else {
                        (false, field_type)
                    };

                    // Map Rust types to SQL types
                    let sql_type = match clean_type.as_str() {
                        "String" => "text",
                        "i32" => "integer",
                        "i64" => "bigint",
                        t if t.starts_with("Id<") => "text",
                        _ => "text", // Default
                    };

                    columns.push(ColumnSnapshot {
                        name: field_name.clone(),
                        ty: sql_type.to_string(),
                        nullable,
                    });

                    if is_key {
                        primary_key.push(field_name.clone());
                        has_key = true;
                    }

                    if is_unique || is_key {
                        indices.push(IndexSnapshot {
                            name: format!("index_{}_by_{}", table_name, field_name),
                            columns: vec![field_name.clone()],
                            unique: true,
                            primary_key: is_key,
                        });
                    } else if is_index {
                        indices.push(IndexSnapshot {
                            name: format!("index_{}_by_{}", table_name, field_name),
                            columns: vec![field_name],
                            unique: false,
                            primary_key: false,
                        });
                    }
                }
            }

            i += 1;
        }

        // Don't add default id - models should always have #[key] field

        Ok(Some(TableSnapshot {
            name: table_name,
            columns,
            indices,
            primary_key,
        }))
    }
}
