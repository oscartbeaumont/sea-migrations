use sea_migrations::run_migrations;
use sea_orm::Database;

mod models;

#[tokio::main]
async fn main() -> Result<(), sea_orm::DbErr> {
    let db = Database::connect("sqlite://./test.db?mode=rwc").await?;

    let migrations_result = run_migrations(&db, models::do_migrations).await;
    if let Err(e) = migrations_result {
        eprintln!("Migration Error: {}", e);
    } else {
        println!("Migrations successful!");
    }

    Ok(())
}
