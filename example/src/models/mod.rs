use sea_migrations::{create_entity_table, MigrationStatus};
use sea_orm::{
    sea_query::{Alias, ColumnDef, Table},
    ConnectionTrait, DbConn, DbErr,
};

pub mod customer;

/// do_migrations is the callback for sea-migrations to run the migrations
pub async fn do_migrations(
    db: &DbConn,
    current_migration_version: Option<u32>,
) -> Result<MigrationStatus, DbErr> {
    match current_migration_version {
        None => {
            println!("Migrating empty DB -> version 1!");
            create_entity_table(db, customer::Entity).await?;
            Ok(MigrationStatus::Complete)
        }
        Some(1) => {
            println!("Migrating from version 1 -> 2!");
            let stmt = Table::alter()
                .table(customer::Entity)
                .add_column(
                    ColumnDef::new(Alias::new("new_column"))
                        .integer()
                        .not_null()
                        .default(100),
                )
                .to_owned();
            db.execute(db.get_database_backend().build(&stmt)).await?;

            Ok(MigrationStatus::Complete)
        }
        Some(2) => Ok(MigrationStatus::NotRequired),
        _ => Err(DbErr::Custom(":(".into())),
    }
}
