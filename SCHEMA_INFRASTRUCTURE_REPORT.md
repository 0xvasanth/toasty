# Toasty Schema Infrastructure Exploration Report

## Executive Summary

This report documents the current schema infrastructure in Toasty that migrations will need to work with. The infrastructure is built on a clear separation between **application schema** (app models) and **database schema** (physical tables), with automatic mapping between them. Currently, there is no versioning or change tracking system—schemas are built fresh from model definitions on each application startup.

---

## 1. Schema Architecture Overview

### 1.1 Two-Level Schema System

Toasty maintains two distinct schema representations:

#### **Application Schema (`toasty-core/src/schema/app/`)**
- **Purpose**: Represents application-level model definitions (user-written code)
- **Key Components**:
  - `Model`: Application model with fields, relations, constraints
  - `Field`/`FieldTy`: Supports Primitive, BelongsTo, HasMany, HasOne types
  - `Schema`: Collection of all models
  - `Index`: Defines primary keys and secondary indexes
- **Type System**: `stmt::Type` (internal application types like I8, I16, I32, I64, String, Bool, Uuid, etc.)
- **Features**:
  - Fields can have `storage_name` different from `app_name`
  - Fields can have `storage_ty` hints (e.g., VARCHAR(255) vs TEXT)
  - Relations are bidirectional (HasMany ↔ BelongsTo, HasOne ↔ BelongsTo)
  - Nullable, primary_key, auto-increment flags
  - Constraints (e.g., length requirements)

#### **Database Schema (`toasty-core/src/schema/db/`)**
- **Purpose**: Represents actual physical database tables and columns
- **Key Components**:
  - `Table`: Physical database table with name, columns, indices
  - `Column`: Database column with name, type, nullable flag
  - `Schema`: Collection of all tables
  - `Type`: Database storage types (Boolean, Integer(n), Text, VarChar(n), Uuid, Custom)
- **Type System**: `db::Type` (database-specific storage types)
- **Features**:
  - Simple flat structure (tables contain columns)
  - Column types map directly to database capabilities
  - No inherent relationships (foreign keys are implicit in app schema)

#### **Mapping Layer (`toasty-core/src/schema/mapping/`)**
- **Purpose**: Links application models to database tables
- **Key Components**:
  - `Mapping`: Container mapping each ModelId → Model (database representation)
  - `mapping::Model`: Maps app model to table with field→column mappings
  - `TableToModel`: Expression for converting table rows back to app model
  - `Field`: Maps app field to column with lowering information
- **Features**:
  - One model can map to one table
  - Multiple models can reference same table (with different column subsets)
  - Stores expressions for bidirectional conversion

### 1.2 Complete Schema Structure

```
Schema (in toasty-core)
├── app: app::Schema
│   └── models: IndexMap<ModelId, Model>
│       ├── id: ModelId (runtime-generated)
│       ├── name: Name (can be multi-part)
│       ├── table_name: Option<String> (optional explicit mapping)
│       ├── fields: Vec<Field>
│       │   ├── id: FieldId
│       │   ├── name: FieldName (app_name + optional storage_name)
│       │   ├── ty: FieldTy (Primitive | BelongsTo | HasMany | HasOne)
│       │   ├── nullable: bool
│       │   ├── primary_key: bool
│       │   ├── auto: Option<Auto> (Auto::Id)
│       │   └── constraints: Vec<Constraint>
│       ├── primary_key: PrimaryKey (fields + index reference)
│       └── indices: Vec<Index>
│           ├── fields: Vec<IndexField>
│           ├── unique: bool
│           └── primary_key: bool
├── db: Arc<db::Schema>
│   └── tables: Vec<Table>
│       ├── id: TableId
│       ├── name: String
│       ├── columns: Vec<Column>
│       │   ├── id: ColumnId
│       │   ├── name: String
│       │   ├── ty: stmt::Type (app-level type)
│       │   ├── storage_ty: Option<db::Type> (physical storage hint)
│       │   ├── nullable: bool
│       │   └── primary_key: bool
│       ├── primary_key: PrimaryKey
│       └── indices: Vec<Index>
└── mapping: Mapping
    └── models: IndexMap<ModelId, mapping::Model>
        ├── id: ModelId
        ├── table: TableId
        ├── columns: Vec<ColumnId>
        ├── fields: Vec<Option<mapping::Field>>
        ├── model_to_table: stmt::ExprRecord
        ├── model_pk_to_table: stmt::Expr
        └── table_to_model: TableToModel
```

---

## 2. Schema Definition and Representation

### 2.1 Application Schema Construction

**Source**: Models are defined via the `#[derive(Model)]` macro

```rust
#[derive(Model)]
#[table("users")]  // Optional explicit table mapping
struct User {
    #[key]
    #[auto]
    id: Id<Self>,
    
    #[index]  // Creates secondary index
    name: String,
    
    #[unique]  // Creates unique index
    email: String,
}
```

**Flow**:
1. **Macro Processing** (`toasty-macros`)
   - Parses struct attributes and field attributes
   - Generates unique `ModelId` via `generate_unique_id()`

2. **Code Generation** (`toasty-codegen`)
   - `schema/model.rs`: Parses model attributes, builds structure
   - `expand/schema.rs`: Generates `schema()` method returning `app::Model`
   - Each model's `schema()` method is called at runtime during DB initialization

3. **Schema Building** (`toasty-core/src/schema/builder.rs`)
   - Collects all model definitions
   - Verifies each model against driver capabilities
   - Builds table stubs and creates mappings
   - Builds table columns from model fields
   - Links relations (HasMany ↔ BelongsTo pairs)
   - Performs final verification

### 2.2 Field Type Mapping

**Primitive Types** (`stmt::Type`):
```
I8, I16, I32, I64, U8, U16, U32, U64
Bool, String, Uuid
Id<Model>  // Model-specific IDs
```

**Storage Type Mapping** (`db::Type`):
```
Primitive::String
  → db::Text (default)
  → db::VarChar(n) (if specified via #[column(ty = "...")]

Primitive::I32
  → db::Integer(4)

Primitive::Bool
  → db::Boolean
```

**Type Hints**: Fields can specify storage type via macro:
```rust
#[column(ty = db::VarChar(255))]
name: String,  // Stored as VARCHAR(255) not TEXT
```

### 2.3 Index Representation

**Primary Keys**:
- Defined via `#[key]` on fields or `#[key(partition = ["id"])]` on struct
- Can be composite
- Stored as PrimaryKey struct with field references and index reference

**Secondary Indices**:
- `#[index]`: Creates non-unique index
- `#[unique]`: Creates unique index
- Stored in `model.indices` Vec

**Index Scopes** (for DynamoDB):
- `Partition`: Partition key component
- `Local`: Local sort key component (for OLAP queries)

### 2.4 Relations

**BelongsTo**: Many-to-one relationship
```rust
struct Comment {
    #[key] id: Id<Self>,
    #[belongs_to(Post)]  // Foreign key reference
    post_id: Id<Post>,
}
```
- Generates foreign key field(s)
- Links to HasMany/HasOne on other side

**HasMany**: One-to-many collection
```rust
struct Post {
    #[has_many(Comment)]
    comments: Vec<Comment>,
}
```
- Virtual field (no column storage)
- Pairs with BelongsTo

**HasOne**: One-to-one relationship
```rust
struct Post {
    #[has_one(Author)]
    author: Author,
}
```
- Virtual field
- Pairs with BelongsTo

---

## 3. Schema Change Detection

### 3.1 Current Approach: No Change Detection

**Important**: Toasty has **no migration system** currently. The approach is:

1. Models are defined in code via `#[derive(Model)]`
2. `app::Schema::from_macro()` is called during DB initialization
3. Driver's `reset_db()` is called to drop/recreate all tables
4. This happens **every time** the application starts in test mode

**Code Path**:
```
toasty/src/db/builder.rs: Builder::build()
  └─ app::Schema::from_macro(&self.models)
  └─ schema::Builder::build(app_schema, driver_capability)
  └─ driver.register_schema(&schema.db)
  └─ (in tests) db.reset_db() calls driver.reset_db(&schema.db)
```

### 3.2 Schema Change Detection Opportunities

For a migration system, these are the points where changes could be detected:

**Option A: At Build Time**
- Compare incoming `app::Schema` with persisted schema
- Requires: Schema serialization/persistence layer (new)

**Option B: At Driver Level**
- Driver introspects database schema
- Compares with target `db::Schema`
- Generates migration plan
- Driver implementations already have partial introspection:
  - `toasty-driver-sqlite/src/lib.rs`: create_table() method
  - `toasty-driver-postgresql/src/lib.rs`: create_table(), drop_table() methods
  - Uses SQL serialization for generation

**Option C: Hybrid**
- Persist schema version/hash
- Driver compares current DB schema with expected
- Generate minimal migrations

### 3.3 Current Schema Access Points

**In Application**:
```rust
let db = Db::builder()
    .register::<User>()
    .register::<Post>()
    .connect("sqlite::memory:")
    .await?;

// Access schema:
let schema = db.schema();  // Returns &Schema
// Access app schema: schema.app
// Access db schema: schema.db (Arc<db::Schema>)
// Access mapping: schema.mapping
```

**Schema Components Available**:
```rust
pub struct Schema {
    pub app: app::Schema,                    // App-level models
    pub db: Arc<db::Schema>,                 // DB tables
    pub mapping: Mapping,                    // App ↔ DB mapping
}
```

---

## 4. Driver Schema Operations

### 4.1 Driver Trait Interface

**Location**: `toasty-core/src/driver.rs`

```rust
#[async_trait]
pub trait Driver: Debug + Send + Sync + 'static {
    /// Describes the driver's capability
    fn capability(&self) -> &Capability;

    /// Register the schema with the driver
    async fn register_schema(&mut self, schema: &Schema) -> Result<()>;

    /// Execute a database operation
    async fn exec(&self, schema: &Arc<Schema>, plan: Operation) -> Result<Response>;

    /// Reset database (drop and recreate all tables)
    async fn reset_db(&self, schema: &Schema) -> Result<()>;
}
```

### 4.2 Current Implementation Pattern

All drivers implement `register_schema()` as a no-op:
```rust
async fn register_schema(&mut self, _schema: &Schema) -> Result<()> {
    Ok(())
}
```

This is intentional—registration happens via explicit `reset_db()` calls in tests.

### 4.3 Driver-Specific Schema Operations

**SQLite** (`crates/toasty-driver-sqlite/src/lib.rs`):
```rust
fn create_table(&self, schema: &Schema, table: &Table) -> Result<()> {
    let serializer = sql::Serializer::sqlite(schema);
    let mut params = vec![];
    let stmt = serializer.serialize(
        &sql::Statement::create_table(table, &Capability::SQLITE),
        &mut params,
    );
    connection.execute(&stmt, [])?;
    
    // Also creates indices
    for index in &table.indices {
        if !index.primary_key {
            let stmt = serializer.serialize(
                &sql::Statement::create_index(index),
                &mut params,
            );
            connection.execute(&stmt, [])?;
        }
    }
    Ok(())
}

async fn reset_db(&self, schema: &Schema) -> Result<()> {
    for table in &schema.tables {
        self.create_table(schema, table)?;
    }
    Ok(())
}
```

**PostgreSQL** (`crates/toasty-driver-postgresql/src/lib.rs`):
```rust
pub async fn create_table(&self, schema: &Schema, table: &Table) -> Result<()> {
    let serializer = sql::Serializer::postgresql(schema);
    let mut params = Vec::new();
    let sql = serializer.serialize(
        &sql::Statement::create_table(table, &Capability::POSTGRESQL),
        &mut params,
    );
    self.client.execute(&sql, &[]).await?;
    
    // Create indices
    for index in &table.indices {
        if !index.primary_key {
            let sql = serializer.serialize(
                &sql::Statement::create_index(index),
                &mut params,
            );
            self.client.execute(&sql, &[]).await?;
        }
    }
    Ok(())
}

pub async fn drop_table(&self, schema: &Schema, table: &Table, if_exists: bool) -> Result<()> {
    let serializer = sql::Serializer::postgresql(schema);
    let mut params = Vec::new();
    
    let sql = if if_exists {
        serializer.serialize(&sql::Statement::drop_table_if_exists(table), &mut params)
    } else {
        serializer.serialize(&sql::Statement::drop_table(table), &mut params)
    };
    
    self.client.execute(&sql, &[]).await?;
    Ok(())
}

async fn reset_db(&self, schema: &Schema) -> Result<()> {
    for table in &schema.tables {
        self.drop_table(schema, table, true).await?;
        self.create_table(schema, table).await?;
    }
    Ok(())
}
```

**MySQL**: Similar pattern to PostgreSQL
**DynamoDB**: Create tables via DynamoDB API (not SQL)

### 4.4 SQL Serialization for DDL

**Location**: `crates/toasty-sql/src/stmt/create_table.rs`

```rust
pub fn create_table(table: &Table, capability: &Capability) -> Statement {
    CreateTable {
        table: table.id,
        columns: table.columns.iter()
            .map(|column| ColumnDef::from_schema(column, &capability.storage_types))
            .collect(),
        primary_key: Some(Box::new(stmt::Expr::record(
            table.primary_key.columns.iter()
                .map(|col| stmt::Expr::column(*col))
        ))),
    }.into()
}
```

**Column Definition**:
```rust
pub struct ColumnDef {
    pub column: ColumnId,
    pub name: String,
    pub ty: db::Type,  // Storage type
    pub nullable: bool,
    pub primary_key: bool,
}

impl ColumnDef {
    pub fn from_schema(column: &Column, storage_types: &StorageTypes) -> Self {
        let ty = db::Type::from_app(
            &column.ty,
            &column.storage_ty,
            storage_types
        )?;
        // ...
    }
}
```

**Generated SQL** (example):
```sql
CREATE TABLE users (
    id TEXT NOT NULL,
    name VARCHAR(255) NOT NULL,
    email TEXT NOT NULL,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX unique_email ON users(email);
```

---

## 5. Schema Verification

**Location**: `toasty-core/src/schema/verify.rs`

Built-in verification checks:

1. **IDs Populated**: All ModelId, FieldId, TableId, ColumnId are valid
2. **Relations Indexed**: Foreign key fields have proper indices
3. **Index Scoping**: Index fields correctly ordered (partition before local)
4. **Index Columns**: All indices have at least one column
5. **Index Names**: All index names unique across all tables
6. **Table Indices**: Multi-column indices have no nullable columns

Verification is called after schema building:
```rust
let schema = Schema { app, db, mapping };
schema.verify();  // Panics on failures
```

---

## 6. Schema Workflow

### 6.1 Complete Schema Initialization Flow

```
User Code
  ↓
#[derive(Model)] macro
  ├─ Parses struct attributes
  ├─ Generates Model structs
  └─ Generates Model::schema() method returning app::Model
  ↓
Db::builder().register::<User>().connect(...).await
  ↓
Builder::build(driver)
  ├─ Calls User::schema() for each registered model
  ├─ Builds app::Schema from models
  |   └─ app::Schema::from_macro(&[model1, model2, ...])
  |       └─ Links relations (HasMany ↔ BelongsTo pairs)
  ├─ Builds db::Schema via schema::Builder::build()
  |   ├─ Verifies each model against driver capability
  |   ├─ Builds table stubs from models
  |   ├─ Creates mapping entries
  |   ├─ Builds columns from fields
  |   ├─ Creates indices
  |   └─ Verifies complete schema structure
  ├─ Calls driver.register_schema(&db::Schema)
  |   └─ Currently a no-op
  ├─ Creates Engine with Arc<Schema>
  └─ Returns Db
  ↓
(In tests) db.reset_db()
  └─ Calls driver.reset_db(&schema.db)
      └─ Drops and recreates all tables via SQL
  ↓
Application ready
```

### 6.2 Schema Access During Runtime

```rust
// Access from Db
let schema = db.schema();  // &Schema

// Application schema
schema.app.models();       // Iterator over app::Models
schema.app.model(id);      // Get specific app::Model
schema.app.field(id);      // Get specific app::Field

// Database schema
schema.db.tables;          // Vec<Table>
schema.db.table(id);       // Get specific Table
schema.db.column(id);      // Get specific Column

// Mapping
schema.mapping.models;     // Map of ModelId → mapping::Model
schema.mapping.model(id);  // Get mapping for app::Model
```

---

## 7. Type System and Storage Mapping

### 7.1 Type Flow

```
Rust Type (in model definition)
  ↓
stmt::Type (application/query engine type)
  I8, I16, I32, I64 (signed integers)
  U8, U16, U32, U64 (unsigned integers)
  Bool, String, Uuid, Id<Model>
  ↓
db::Type (database storage type)
  Integer(1), Integer(2), Integer(4), Integer(8)  [signed]
  UnsignedInteger(1), UnsignedInteger(2), UnsignedInteger(4), UnsignedInteger(8)
  Boolean, Text, VarChar(n), Uuid, Custom(String)
  ↓
Native Database Type
  PostgreSQL: INT2, INT4, INT8, BOOLEAN, TEXT, VARCHAR(n), UUID
  SQLite: INTEGER (affinity), TEXT, BLOB (for UUID)
  MySQL: TINYINT, SMALLINT, INT, BIGINT, BOOL, VARCHAR(n), TEXT, BINARY(16)
  DynamoDB: N (number), S (string), BOOL
```

### 7.2 Type Mapping Defaults

**From `db::Type::from_app()`**:
```rust
stmt::Type::Bool         → db::Type::Boolean
stmt::Type::I8           → db::Type::Integer(1)
stmt::Type::I16          → db::Type::Integer(2)
stmt::Type::I32          → db::Type::Integer(4)
stmt::Type::I64          → db::Type::Integer(8)
stmt::Type::U8           → db::Type::UnsignedInteger(1)
stmt::Type::U16          → db::Type::UnsignedInteger(2)
stmt::Type::U32          → db::Type::UnsignedInteger(4)
stmt::Type::U64          → db::Type::UnsignedInteger(8)
stmt::Type::String       → db.storage_types.default_string_type
  (PostgreSQL: Text, MySQL: VarChar(255), SQLite: Text)
stmt::Type::Uuid         → db::Type::Uuid
stmt::Type::Id<Model>    → db.storage_types.default_string_type
```

### 7.3 Storage Type Customization

Field-level storage type hints:
```rust
#[column(ty = db::VarChar(100))]
short_text: String,  // Stored as VARCHAR(100)

#[column(ty = db::Integer(1))]
small_number: i8,    // Explicitly 1-byte integer
```

---

## 8. Schema Constraints and Verification

### 8.1 Field-Level Constraints

```rust
// Automatically created from storage type limits
#[column(ty = db::VarChar(255))]
name: String,  // Adds length constraint: ≤ 255 chars
```

These become `Constraint::LengthLessThan(n)` in the schema.

### 8.2 Relation Constraints

- Foreign key fields must exist and be indexed
- Foreign key must point to primary key of target model
- HasMany/BelongsTo pairs must exist and be bidirectional
- Indices on foreign keys must be single-column (currently)

---

## 9. Database Capability System

**Location**: `toasty-core/src/driver/capability.rs`

Each driver advertises supported features:

```rust
pub struct Capability {
    pub storage_types: StorageTypes,  // Supported column types
}

pub struct StorageTypes {
    pub default_string_type: db::Type,  // TEXT vs VARCHAR default
    pub varchar: Option<u64>,           // Max VARCHAR length, if supported
    pub uuid: Option<db::Type>,         // UUID support
}
```

**Examples**:
```rust
Capability::SQLITE {
    storage_types: StorageTypes {
        default_string_type: db::Type::Text,
        varchar: Some(100000),  // SQLite supports VARCHAR
        uuid: None,  // Stored as BLOB or TEXT
    }
}

Capability::POSTGRESQL {
    storage_types: StorageTypes {
        default_string_type: db::Type::Text,
        varchar: Some(u32::MAX),  // PostgreSQL VARCHAR(n) supported
        uuid: Some(db::Type::Uuid),  // Native UUID
    }
}
```

---

## 10. Key Findings for Migration Design

### 10.1 Strengths

1. **Comprehensive Schema Representation**
   - Full app and db schema captured
   - Clear mapping between application and database levels
   - All necessary information available for comparison

2. **Flexibility**
   - Models can map to custom table names
   - Fields can have different storage names
   - Storage types can be customized per field
   - Multi-model single-table mapping supported

3. **Driver Abstraction**
   - Clean Driver trait interface
   - SQL serialization already exists for CREATE/DROP
   - Capability system allows DB-specific behavior

4. **Verification System**
   - Comprehensive schema validation
   - Would catch invalid migration states

### 10.2 Gaps for Migrations

1. **No Schema Persistence**
   - Current schemas only exist in memory
   - No way to retrieve "current database schema" state
   - No version tracking

2. **No Change Detection**
   - No mechanism to compare old vs new schema
   - No diff/patch generation
   - No rollback support

3. **Limited DDL Support**
   - Only CREATE TABLE and CREATE INDEX
   - No ALTER TABLE, DROP COLUMN, etc.
   - No foreign key handling (implicit in app schema)

4. **No Migration Storage**
   - No "migrations applied" tracking in database
   - No versioning metadata table
   - No rollback history

### 10.3 Required Infrastructure for Migrations

1. **Schema Serialization** (new)
   - Serialize app::Schema and db::Schema to persistent format
   - Git-friendly format (JSON, YAML, or custom)

2. **Change Detection** (new)
   - Compare old vs new schemas
   - Generate list of changes (add field, drop field, rename, etc.)
   - Track change history

3. **Migration Generation** (new)
   - Auto-generate migration from schema changes
   - SQL statement generation for each change
   - Validation that changes are safe

4. **Migration Execution** (new)
   - Track which migrations have been applied
   - Idempotent execution
   - Rollback capability

5. **DDL Statement Generation** (extends existing)
   - ALTER TABLE for field changes
   - DROP COLUMN for removed fields
   - CREATE/DROP FOREIGN KEY constraints
   - Extend `toasty-sql` with more statement types

---

## 11. Code Organization Reference

```
crates/toasty-core/src/schema/
├── app/                          # Application-level schema
│   ├── model.rs                 # Model, ModelId, PrimaryKey
│   ├── field.rs                 # Field, FieldTy, FieldName
│   ├── field/primitive.rs       # FieldPrimitive
│   ├── schema.rs                # app::Schema, builder
│   ├── relation/                # HasMany, HasOne, BelongsTo
│   ├── index.rs                 # Index definition
│   └── constraint.rs            # Field constraints
├── db/                           # Database-level schema
│   ├── table.rs                 # Table, TableId
│   ├── column.rs                # Column, ColumnId
│   ├── schema.rs                # db::Schema
│   ├── ty.rs                    # db::Type (storage types)
│   ├── index.rs                 # Index (database indices)
│   └── pk.rs                    # PrimaryKey
├── mapping/                      # App ↔ DB mapping
│   ├── model.rs                 # mapping::Model, TableToModel
│   └── field.rs                 # mapping::Field
├── builder.rs                    # Schema builder (app → db transformation)
├── name.rs                       # Name type (multi-part names)
├── verify.rs                     # Schema verification
└── mod.rs                        # Schema container, Schema type

crates/toasty-sql/src/
├── stmt/
│   ├── create_table.rs          # CreateTable statement
│   ├── create_index.rs          # CreateIndex statement
│   └── ...
└── serializer/
    ├── statement.rs              # Convert statements to SQL strings

crates/toasty-driver-*/src/
├── lib.rs                        # Driver impl with reset_db, create_table
└── value.rs                      # Type conversions to/from native DB types

crates/toasty/src/
├── db/
│   ├── builder.rs               # Db initialization, register_schema call
│   └── mod.rs                   # Db struct
├── schema.rs                     # Re-exports from toasty-core
└── driver.rs                     # Multi-driver wrapper
```

---

## 12. Testing Infrastructure Notes

Current test setup (`tests/tests/`):

```rust
test.setup_db(models!(User, Post)).await
  └─ Creates temporary database
  └─ Builds schema from models
  └─ Calls reset_db() to create all tables
  └─ Ready for testing

db.reset_db().await  // Drop and recreate all tables
```

This shows that tables are created fresh each test—a migration system would need to handle:
- Preserving existing data
- Incremental changes
- Schema versioning

---

## Conclusion

Toasty's schema infrastructure is well-designed and comprehensive. It provides:

1. Clear separation between application models and database schema
2. Type-safe schema representation with proper mapping
3. Driver abstraction that supports multiple databases
4. Mechanisms for schema verification and validation

To implement migrations, the system would need to add:

1. **Persistence layer**: Schema snapshots and change tracking
2. **Diff engine**: Compare old vs new schemas
3. **Migration generator**: Create SQL DDL from diffs
4. **Execution tracker**: Record and apply migrations
5. **Extended DDL support**: ALTER TABLE, DROP COLUMN, etc.

The existing infrastructure provides an excellent foundation for these additions.
