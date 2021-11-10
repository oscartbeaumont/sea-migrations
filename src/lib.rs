#![deny(missing_docs)]

//! Effortless database migrations for [SeaORM](https://www.sea-ql.org/SeaORM/)!
//!
//! Checkout an example using this package [here](https://github.com/oscartbeaumont/sea-migrations/tree/main/example).

use std::future::Future;

use sea_orm::{
    sea_query::{ColumnDef, Table},
    ColumnTrait, ColumnType, ConnectionTrait, DbConn, DbErr, EntityTrait, ExecResult, Iterable,
    PrimaryKeyToColumn, PrimaryKeyTrait,
};

mod migrations_table;

/// MigrationStatus is used to represent the status of a migration.
#[derive(PartialEq)]
pub enum MigrationStatus {
    /// NotRequired is returned when no database migrations are required. If this is returned from the database migrations callback it is assumed by sea-migrations that the database is already up to date.
    NotRequired,
    /// Complete is returned when a migration has been run successfully. This will cause a new migration event to be added and the migration version to be incremented.
    Complete,
}

/// run_migrations will run the database migrations. It takes in callback function which will be called to do the migrations.
///
/// ```rust
/// use sea_migrations::{run_migrations, create_entity_table, MigrationStatus};
/// use sea_orm::{ Database, DbErr, ConnectionTrait, DbConn};
/// use sea_orm::sea_query::{Alias, ColumnDef, Table};
///
/// pub async fn migrations_callback(db: &DbConn, current_migration_version: Option<u32>) -> Result<MigrationStatus, DbErr> {
///     match current_migration_version {
///         None => Ok(MigrationStatus::NotRequired), // Tells sea-migrations that no further migrations are required.
///         _ => Err(DbErr::Custom("Invalid migrations version number!".into())),
///     }
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), sea_orm::DbErr> {
///     let db = Database::connect("sqlite::memory:").await?;
///     let migrations_result = run_migrations(&db, migrations_callback).await?;
///     Ok(())
/// }
///
/// ```
pub async fn run_migrations<'a, T, F>(db: &'a DbConn, handler: F) -> Result<MigrationStatus, DbErr>
where
    T: Future<Output = Result<MigrationStatus, DbErr>>,
    F: Fn(&'a DbConn, Option<u32>) -> T,
{
    migrations_table::init(db).await?;
    let current_migrations_version = migrations_table::get_latest(db).await?;
    let result = handler(db, current_migrations_version).await;
    if result == Ok(MigrationStatus::Complete) {
        migrations_table::insert_migration(db).await?;
    }

    result
}

// CustomColumnDef is a copy of the struct defined at https://github.com/SeaQL/sea-query/blob/master/src/table/column.rs#L5 with all fields set to public.
// It exists so that the unsafe transmutate operation can be applied to access private fields on the struct.
// This is a TEMPORARY solution and I will ask if these values can be directly exposed by sea_query in the future. This solution relies on internal implementation details of sea_query and unsafe code which is not good!
struct CustomColumnDef {
    pub col_type: ColumnType,
    pub null: bool,
    pub unique: bool,
    pub indexed: bool,
}

/// create_entity_table will automatically create a database table if it does not exist for a sea_query Entity.
///
/// ```rust
/// use sea_orm::{Database, DbErr, ConnectionTrait, DbConn};
/// use sea_orm::entity::prelude::*;
/// use sea_migrations::create_entity_table;
///
/// #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
/// #[sea_orm(table_name = "cake")]
/// pub struct Model {
///     #[sea_orm(primary_key)]
///     pub id: i32,
///     pub name: String,
/// }

/// #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
/// pub enum Relation {}
///
/// impl ActiveModelBehavior for ActiveModel {}
///
/// #[tokio::main]
/// async fn main() -> Result<(), sea_orm::DbErr> {
///     let db = Database::connect("sqlite::memory:").await?;
///
///     create_entity_table(&db, crate::Entity).await?; // Replace "crate" with the name of the module containing your SeaORM Model.
///
///     Ok(())
/// }
///
/// ```
pub async fn create_entity_table<T: 'static>(db: &DbConn, entity: T) -> Result<ExecResult, DbErr>
where
    T: EntityTrait,
{
    let mut stmt = Table::create();
    stmt.table(entity).if_not_exists();

    for column in T::Column::iter() {
        let column_def_prelude: CustomColumnDef = unsafe { std::mem::transmute(column.def()) }; // Note: This is used to access private fields and hence relies on internal implementation details of sea_query and unsafe code which is not good!
        let column_def =
            &mut ColumnDef::new_with_type(column, column_def_prelude.col_type.clone().into());
        if column_def_prelude.null {
            column_def.not_null();
        }
        if column_def_prelude.unique {
            column_def.unique_key();
        }
        if column_def_prelude.indexed {
            panic!("Indexed columns are not yet able to be migrated!");
        }

        if let Some(_) = T::PrimaryKey::from_column(column) {
            column_def.primary_key();

            if T::PrimaryKey::auto_increment() && column_def_prelude.col_type == ColumnType::Integer
            {
                column_def.auto_increment();
            }
        }

        stmt.col(column_def);
    }

    db.execute(db.get_database_backend().build(&stmt)).await
}
