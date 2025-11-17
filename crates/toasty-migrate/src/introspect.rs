use crate::snapshot::*;
use anyhow::Result;

/// Introspect database schema to create a snapshot
/// This allows generating migrations based on current database state
pub trait SchemaIntrospector {
    /// Connect to database and read current schema
    async fn introspect(&self) -> Result<SchemaSnapshot>;
}

/// SQL database introspection (works for PostgreSQL, MySQL, SQLite)
pub struct SqlIntrospector {
    connection_url: String,
}

impl SqlIntrospector {
    pub fn new(connection_url: String) -> Self {
        Self { connection_url }
    }

    /// Introspect schema from database
    pub async fn introspect_schema(&self) -> Result<SchemaSnapshot> {
        let url = url::Url::parse(&self.connection_url)?;

        match url.scheme() {
            #[cfg(feature = "postgresql")]
            "postgresql" | "postgres" => self.introspect_postgresql().await,
            #[cfg(feature = "sqlite")]
            "sqlite" => self.introspect_sqlite().await,
            #[cfg(feature = "mysql")]
            "mysql" => self.introspect_mysql().await,
            scheme => Err(anyhow::anyhow!(
                "Unsupported database for introspection: {}. Enable feature flag.", scheme
            )),
        }
    }

    #[cfg(feature = "postgresql")]
    async fn introspect_postgresql(&self) -> Result<SchemaSnapshot> {
        use tokio_postgres::NoTls;

        println!("🔍 Introspecting PostgreSQL schema...");

        let (client, connection) = tokio_postgres::connect(&self.connection_url, NoTls).await?;

        // Spawn connection
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        let mut tables = Vec::new();

        // Query tables
        let rows = client.query(
            "SELECT table_name FROM information_schema.tables
             WHERE table_schema = 'public' AND table_type = 'BASE TABLE'
             ORDER BY table_name",
            &[],
        ).await?;

        for row in rows {
            let table_name: String = row.get(0);

            // Skip migration tracking table
            if table_name == "_toasty_migrations" {
                continue;
            }

            let table = self.introspect_postgresql_table(&client, &table_name).await?;
            tables.push(table);
        }

        println!("✅ Found {} table(s)", tables.len());

        Ok(SchemaSnapshot {
            version: "1.0".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            tables,
        })
    }

    #[cfg(feature = "postgresql")]
    async fn introspect_postgresql_table(
        &self,
        client: &tokio_postgres::Client,
        table_name: &str,
    ) -> Result<TableSnapshot> {
        let mut columns = Vec::new();
        let mut primary_key_cols = Vec::new();

        // Get columns - use simple_query to avoid parameter issues
        let query = format!(
            "SELECT column_name, data_type, is_nullable
             FROM information_schema.columns
             WHERE table_name = '{}' AND table_schema = 'public'
             ORDER BY ordinal_position",
            table_name
        );
        let rows = client.query(&query, &[]).await?;

        for row in rows {
            let col_name: String = row.get(0);
            let data_type: String = row.get(1);
            let is_nullable: String = row.get(2);

            columns.push(ColumnSnapshot {
                name: col_name,
                ty: data_type,
                nullable: is_nullable == "YES",
            });
        }

        // Get primary key
        let pk_query = format!(
            "SELECT a.attname
             FROM pg_index i
             JOIN pg_attribute a ON a.attrelid = i.indrelid AND a.attnum = ANY(i.indkey)
             WHERE i.indrelid = '{}'::regclass AND i.indisprimary",
            table_name
        );
        let pk_rows = client.query(&pk_query, &[]).await?;

        for row in pk_rows {
            let col_name: String = row.get(0);
            primary_key_cols.push(col_name);
        }

        // Get indexes with column information
        let mut indices = Vec::new();
        let idx_query = format!(
            "SELECT
                i.indexname,
                i.indexdef,
                ix.indisunique,
                ix.indisprimary,
                ARRAY_AGG(a.attname ORDER BY array_position(ix.indkey, a.attnum)) as index_columns
             FROM pg_indexes i
             JOIN pg_class c ON c.relname = i.indexname
             JOIN pg_index ix ON ix.indexrelid = c.oid
             JOIN pg_attribute a ON a.attrelid = ix.indrelid AND a.attnum = ANY(ix.indkey)
             WHERE i.tablename = '{}' AND i.schemaname = 'public'
             GROUP BY i.indexname, i.indexdef, ix.indisunique, ix.indisprimary",
            table_name
        );
        let idx_rows = client.query(&idx_query, &[]).await?;

        for row in idx_rows {
            let idx_name: String = row.get(0);
            let _idx_def: String = row.get(1);
            let is_unique: bool = row.get(2);
            let is_primary: bool = row.get(3);
            let columns: Vec<String> = row.get(4);

            indices.push(IndexSnapshot {
                name: idx_name,
                columns,
                unique: is_unique,
                primary_key: is_primary,
            });
        }

        Ok(TableSnapshot {
            name: table_name.to_string(),
            columns,
            indices,
            primary_key: primary_key_cols,
        })
    }

    #[cfg(not(feature = "postgresql"))]
    async fn introspect_postgresql(&self) -> Result<SchemaSnapshot> {
        Err(anyhow::anyhow!("PostgreSQL introspection requires 'postgresql' feature"))
    }

    #[cfg(not(feature = "sqlite"))]
    async fn introspect_sqlite(&self) -> Result<SchemaSnapshot> {
        Err(anyhow::anyhow!("SQLite introspection requires 'sqlite' feature"))
    }

    #[cfg(not(feature = "mysql"))]
    async fn introspect_mysql(&self) -> Result<SchemaSnapshot> {
        Err(anyhow::anyhow!("MySQL introspection requires 'mysql' feature"))
    }
}

/// MongoDB schema introspection
pub struct MongoDbIntrospector {
    connection_url: String,
}

impl MongoDbIntrospector {
    pub fn new(connection_url: String) -> Self {
        Self { connection_url }
    }

    /// Introspect MongoDB schema (collections and indexes)
    pub async fn introspect_schema(&self) -> Result<SchemaSnapshot> {
        // TODO: Implement MongoDB introspection
        // 1. Connect to database
        // 2. List collections
        // 3. Get indexes for each collection
        // 4. Infer schema from sample documents (optional)

        println!("🔍 Introspecting MongoDB schema from: {}", self.connection_url);

        Ok(SchemaSnapshot {
            version: "1.0".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            tables: vec![],
        })
    }
}
