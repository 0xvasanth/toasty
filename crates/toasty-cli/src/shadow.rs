use anyhow::Result;
use toasty_migrate::*;
use crate::executor::MigrationExecutor;
use std::path::Path;

/// Extract a quoted string from a line after a prefix
fn extract_quoted_string(line: &str, after: &str) -> Option<String> {
    let start = if after.is_empty() {
        line.find('"')?
    } else {
        line.find(after)? + after.len()
    };

    let remaining = &line[start..];
    let end = remaining.find('"')?;

    Some(remaining[..end].to_string())
}

/// Shadow database for migration diff calculation
/// 
/// Creates a temporary database, applies all existing migrations to it,
/// then introspects to get the "current state after all migrations".
/// This is compared with desired entity schema to generate only new changes.
pub struct ShadowDatabase {
    url: String,
    temp_file: Option<tempfile::NamedTempFile>,
}

impl ShadowDatabase {
    /// Create a new shadow database
    pub fn new() -> Result<Self> {
        // Create temporary SQLite database
        let temp_file = tempfile::NamedTempFile::new()?;
        let shadow_url = format!("sqlite:{}", temp_file.path().display());

        Ok(Self {
            url: shadow_url,
            temp_file: Some(temp_file),
        })
    }

    /// Apply all migrations from directory to shadow database
    pub async fn apply_migrations(&self, migration_dir: &Path) -> Result<SchemaSnapshot> {
        println!("🔄 Creating shadow database...");

        // Load all migration files
        let loader = MigrationLoader::new(migration_dir);
        let migration_files = loader.discover_migrations()?;

        if migration_files.is_empty() {
            println!("   No existing migrations - empty schema");
            return Ok(SchemaSnapshot {
                version: "1.0".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                tables: vec![],
            });
        }

        println!("   Applying {} migration(s) to shadow database", migration_files.len());

        // REAL SHADOW DATABASE APPROACH:
        // We need to execute the actual SQL from each migration
        // Since migration files are .rs code, we need to:
        // 1. Generate SQL from each migration's operations
        // 2. Execute that SQL in the shadow DB
        // 3. Introspect to get real state

        // Parse each migration file to extract SQL operations
        #[cfg(feature = "sqlite")]
        {
            use rusqlite::Connection;
            let shadow_path = self.temp_file.as_ref().unwrap().path();
            let conn = Connection::open(shadow_path)?;

            // Execute each migration by parsing its SQL from the .rs file
            for migration_file in &migration_files {
                let content = std::fs::read_to_string(&migration_file.path)?;

                // Extract SQL from migration file
                let sql_statements = self.extract_sql_from_migration(&content)?;

                for sql in sql_statements {
                    if !sql.trim().is_empty() {
                        println!("      Executing: {}", sql.lines().next().unwrap_or(&sql));
                        conn.execute(&sql, [])?;
                    }
                }
            }

            println!("   ✅ Applied {} migration(s) to shadow database", migration_files.len());

            // Now introspect the shadow database to get real current state
            let introspector = SqlIntrospector::new(self.url.clone());
            let current_state = introspector.introspect_schema().await?;

            println!("   ✅ Shadow database has {} table(s)", current_state.tables.len());

            return Ok(current_state);
        }

        #[cfg(not(feature = "sqlite"))]
        {
            return Err(anyhow::anyhow!("Shadow database requires SQLite feature"));
        }

    }

    /// Extract SQL statements from migration .rs file
    /// Parses Rust code to find db.create_table(), db.add_column(), etc. and converts to SQL
    /// ONLY extracts from up() function, NOT down()
    fn extract_sql_from_migration(&self, content: &str) -> Result<Vec<String>> {
        let mut statements = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        // Find the up() function
        let mut in_up_function = false;
        let mut brace_count = 0;
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // Detect start of up() function
            if line.contains("fn up(&self") {
                in_up_function = true;
                i += 1;
                continue;
            }

            // Track braces to know when we exit up()
            if in_up_function {
                for ch in line.chars() {
                    if ch == '{' {
                        brace_count += 1;
                    } else if ch == '}' {
                        brace_count -= 1;
                        if brace_count == 0 {
                            // Exited up() function
                            in_up_function = false;
                            break;
                        }
                    }
                }
            }

            // Only parse lines inside up() function
            if !in_up_function {
                i += 1;
                continue;
            }

            // Parse db.create_table()
            if line.contains("db.create_table(\"") {
                if let Some(sql) = self.parse_create_table(&lines, i)? {
                    statements.push(sql);
                }
            }
            // Parse db.add_column()
            else if line.contains("db.add_column(\"") {
                if let Some(sql) = self.parse_add_column(line)? {
                    statements.push(sql);
                }
            }
            // Parse db.create_index()
            else if line.contains("db.create_index(\"") {
                if let Some(sql) = self.parse_create_index(line)? {
                    statements.push(sql);
                }
            }
            // Parse db.drop_table()
            else if line.contains("db.drop_table(\"") {
                if let Some(table) = extract_quoted_string(line, "db.drop_table(\"") {
                    statements.push(format!("DROP TABLE IF EXISTS {}", table));
                }
            }
            // Parse db.drop_column()
            else if line.contains("db.drop_column(\"") {
                // Extract table and column names
                if let Some((table, column)) = self.parse_drop_column(line)? {
                    statements.push(format!("ALTER TABLE {} DROP COLUMN {}", table, column));
                }
            }

            i += 1;
        }

        Ok(statements)
    }

    fn parse_create_table(&self, lines: &[&str], start: usize) -> Result<Option<String>> {
        let line = lines[start].trim();

        // Extract table name: db.create_table("users", vec![
        let table_name = extract_quoted_string(line, "db.create_table(\"")
            .ok_or_else(|| anyhow::anyhow!("Failed to parse table name"))?;

        // Parse column definitions
        let mut columns = Vec::new();
        let mut i = start;

        while i < lines.len() {
            let col_line = lines[i].trim();

            // Look for ColumnDef { name: "...", ty: "...", nullable: ... }
            if col_line.contains("ColumnDef {") && col_line.contains("name:") {
                if let Some(col_name) = extract_quoted_string(col_line, "name: \"") {
                    let col_type = extract_quoted_string(col_line, "ty: \"").unwrap_or("TEXT".to_string());
                    let nullable = col_line.contains("nullable: true");

                    let mut col_def = format!("{} {}", col_name, col_type);
                    if !nullable {
                        col_def.push_str(" NOT NULL");
                    }
                    columns.push(col_def);
                }
            }

            // Stop at ])?;
            if col_line.contains("])?;") {
                break;
            }

            i += 1;
        }

        if columns.is_empty() {
            return Ok(None);
        }

        let sql = format!(
            "CREATE TABLE {} (\n  {}\n)",
            table_name,
            columns.join(",\n  ")
        );

        Ok(Some(sql))
    }

    fn parse_add_column(&self, line: &str) -> Result<Option<String>> {
        // db.add_column("users", ColumnDef { name: "bio", ty: "text", nullable: true })?;
        let table = extract_quoted_string(line, "db.add_column(\"")
            .ok_or_else(|| anyhow::anyhow!("Failed to parse table"))?;
        let col_name = extract_quoted_string(line, "name: \"")
            .ok_or_else(|| anyhow::anyhow!("Failed to parse column name"))?;
        let col_type = extract_quoted_string(line, "ty: \"").unwrap_or("TEXT".to_string());
        let nullable = line.contains("nullable: true");

        let mut sql = format!("ALTER TABLE {} ADD COLUMN {} {}", table, col_name, col_type);
        if !nullable {
            sql.push_str(" NOT NULL");
        }

        Ok(Some(sql))
    }

    fn parse_create_index(&self, line: &str) -> Result<Option<String>> {
        // db.create_index("users", IndexDef { name: "idx", columns: vec!["email"], unique: true })?;
        let table = extract_quoted_string(line, "db.create_index(\"")
            .ok_or_else(|| anyhow::anyhow!("Failed to parse table"))?;
        let idx_name = extract_quoted_string(line, "name: \"")
            .ok_or_else(|| anyhow::anyhow!("Failed to parse index name"))?;
        let unique = line.contains("unique: true");

        // Extract columns from vec!["col1", "col2"]
        let columns = if let Some(start) = line.find("columns: vec![") {
            let remaining = &line[start + 14..];
            if let Some(end) = remaining.find("]") {
                let cols_str = &remaining[..end];
                cols_str.split(",")
                    .filter_map(|s| extract_quoted_string(s, "\""))
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        if columns.is_empty() {
            return Ok(None);
        }

        let unique_str = if unique { "UNIQUE " } else { "" };
        let sql = format!("CREATE {}INDEX {} ON {} ({})", unique_str, idx_name, table, columns);

        Ok(Some(sql))
    }

    fn parse_drop_column(&self, line: &str) -> Result<Option<(String, String)>> {
        // db.drop_column("users", "bio")?;
        let table = extract_quoted_string(line, "db.drop_column(\"")
            .ok_or_else(|| anyhow::anyhow!("Failed to parse table"))?;

        // Find second quoted string
        if let Some(first_end) = line.find("\", \"") {
            let remaining = &line[first_end + 4..];
            if let Some(column) = extract_quoted_string(remaining, "") {
                return Ok(Some((table, column)));
            }
        }

        Ok(None)
    }

    /// Get shadow database URL
    pub fn url(&self) -> &str {
        &self.url
    }
}

impl Drop for ShadowDatabase {
    fn drop(&mut self) {
        // Temp file automatically deleted
        println!("🗑️  Shadow database cleaned up");
    }
}
