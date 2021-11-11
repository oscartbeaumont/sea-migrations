use sea_migrations::{add_entity_column, create_entity_table, MigrationStatus};
use sea_orm::{DbConn, DbErr};

pub mod customer;
pub mod customer2;
pub mod tax_info; // Customer -> Tax Info (1:1)

/// do_migrations is the callback for sea-migrations to run the migrations
pub async fn do_migrations(
    db: &DbConn,
    current_migration_version: Option<u32>,
) -> Result<MigrationStatus, DbErr> {
    match current_migration_version {
        None => {
            println!("Migrating empty DB -> version 1!");
            create_entity_table(db, customer::Entity).await?;
            create_entity_table(db, tax_info::Entity).await?;
            Ok(MigrationStatus::Complete)
        }
        Some(1) => {
            println!("Migrating from version 1 -> 2!");
            add_entity_column(db, customer2::Entity, customer2::Column::SomeValue).await?;
            Ok(MigrationStatus::Complete)
        }
        Some(2) => Ok(MigrationStatus::NotRequired),
        _ => Err(DbErr::Custom(":(".into())),
    }
}
