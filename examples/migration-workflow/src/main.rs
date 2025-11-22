use entity::{Post, Role, User, UserRole};
use toasty::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Toasty Migration Workflow Example ===\n");

    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5433/postgres".to_string());

    println!("Connecting to: {}", db_url);

    let db = toasty::Db::builder()
        .register::<User>()
        .register::<Post>()
        .register::<Role>()
        .register::<UserRole>()
        .connect(&db_url)
        .await?;

    db.reset_db().await?;
    println!("Database ready\n");

    let admin = Role::create().name("admin").exec(&db).await?;
    let alice = User::create()
        .name("Alice")
        .username("alice")
        .email("alice@example.com")
        .exec(&db)
        .await?;

    let _post = alice
        .posts()
        .create()
        .title("Hello Toasty")
        .content("Migration example")
        .exec(&db)
        .await?;

    println!("Created: user={}, role={}", alice.name, admin.name);
    Ok(())
}
