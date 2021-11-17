use sea_migrations::Migrator;
use sea_orm::Database;

mod models;

#[tokio::main]
async fn main() -> Result<(), sea_orm::DbErr> {
    let db = Database::connect("sqlite://./test.db?mode=rwc").await?;

    Migrator::run(
        &db,
        &mut vec![
            Box::new(models::M20210101020202DoAThing),
            Box::new(models::M20210105020202DoAThingAgain),
        ],
    )
    .await?;

    Ok(())
}
