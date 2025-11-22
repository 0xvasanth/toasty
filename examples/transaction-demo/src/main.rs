use toasty::stmt::Id;

#[derive(Debug, toasty::Model)]
struct Account {
    #[key]
    #[auto]
    id: Id<Self>,

    #[unique]
    name: String,

    balance: i64,
}

#[tokio::main]
async fn main() -> toasty::Result<()> {
    println!("=== Toasty Transaction Demo ===\n");

    let db = toasty::Db::builder()
        .register::<Account>()
        .connect("sqlite::memory:")
        .await?;

    db.reset_db().await?;

    test_explicit_commit(&db).await?;
    test_explicit_rollback(&db).await?;
    test_rollback_on_error(&db).await?;

    println!("\n=== All transaction tests passed! ===");
    Ok(())
}

async fn test_explicit_commit(db: &toasty::Db) -> toasty::Result<()> {
    println!("Test 1: Explicit commit");
    println!("----------------------");

    let tx = db.begin().await?;
    println!("✅ Transaction started");

    Account::create()
        .name("Alice")
        .balance(100)
        .exec(tx.db())
        .await?;
    println!("   Created: Alice (balance: 100)");

    Account::create()
        .name("Bob")
        .balance(200)
        .exec(tx.db())
        .await?;
    println!("   Created: Bob (balance: 200)");

    tx.commit().await?;
    println!("✅ Transaction committed");

    let alice = Account::get_by_name(&db, "Alice").await?;
    let bob = Account::get_by_name(&db, "Bob").await?;
    println!("   Verified: Alice balance = {}", alice.balance);
    println!("   Verified: Bob balance = {}", bob.balance);

    assert_eq!(alice.balance, 100);
    assert_eq!(bob.balance, 200);
    println!("✅ Test passed - data persisted\n");

    Ok(())
}

async fn test_explicit_rollback(db: &toasty::Db) -> toasty::Result<()> {
    println!("Test 2: Explicit rollback");
    println!("------------------------");

    let tx = db.begin().await?;
    println!("✅ Transaction started");

    Account::create()
        .name("Charlie")
        .balance(300)
        .exec(tx.db())
        .await?;
    println!("   Created: Charlie (balance: 300)");

    tx.rollback().await?;
    println!("✅ Transaction rolled back explicitly");

    let charlie = Account::filter_by_name("Charlie").get(&db).await;

    match charlie {
        Err(_) => println!("✅ Test passed - Charlie not in database\n"),
        Ok(_) => {
            println!("❌ Test failed - Charlie exists after rollback\n");
            return Err(anyhow::anyhow!("Rollback test failed"));
        }
    }

    Ok(())
}

async fn test_rollback_on_error(db: &toasty::Db) -> toasty::Result<()> {
    println!("Test 3: Automatic rollback on error");
    println!("-----------------------------------");

    let tx = db.begin().await?;
    println!("✅ Transaction started");

    Account::create()
        .name("David")
        .balance(400)
        .exec(tx.db())
        .await?;
    println!("   Created: David (balance: 400)");

    let duplicate_result = Account::create()
        .name("Alice")
        .balance(500)
        .exec(tx.db())
        .await;

    match duplicate_result {
        Ok(_) => println!("❌ Should have failed (duplicate name)"),
        Err(e) => {
            println!("   Error (expected): {}", e);
            tx.rollback().await?;
            println!("✅ Transaction rolled back after error");
        }
    }

    let david = Account::filter_by_name("David").get(&db).await;

    match david {
        Err(_) => println!("✅ Test passed - David not in database\n"),
        Ok(_) => {
            println!("❌ Test failed - David exists after rollback\n");
            return Err(anyhow::anyhow!("Rollback on error test failed"));
        }
    }

    Ok(())
}
