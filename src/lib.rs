#![deny(missing_docs)]
// #![deny(unsafe_code)] // TODO: Waiting on https://github.com/SeaQL/sea-orm/issues/317

//! Effortless database migrations for [SeaORM](https://www.sea-ql.org/SeaORM/)!
//!
//! Checkout an example using this package [here](https://github.com/oscartbeaumont/sea-migrations/tree/main/example).

use async_trait::async_trait;
use sea_orm::{
    sea_query::Table, ColumnTrait, ConnectionTrait, DbConn, DbErr, EntityTrait, ExecResult,
    Iterable, RelationTrait,
};

use crate::seaorm_integration::*;
pub use sea_migrations_derive::*;

mod migrations_table;
mod seaorm_integration;

/// MigrationName is the trait implemented on a migration so that sea_migration knows what the migration is called. This is automatically derived by the 'Migration' derive macro.
/// ```rust
/// use sea_migrations::Migration;
///
/// #[derive(Migration)]
/// pub struct M20210101020202DoAThing;
/// ```
pub trait MigrationName {
    /// Returns the name of the migration.
    fn name(&self) -> &'static str;
}

/// MigratorTrait is the trait implemented on a migrator so that sea_migration knows how to do and undo the migration.
///
/// ```rust
/// use sea_orm::DbErr;
/// use sea_migrations::{Migration, MigrationName, MigratorTrait, MigrationManager};
/// use async_trait::async_trait;
///
/// #[derive(Migration)]
/// pub struct M20210101020202DoAThing;
///
/// #[async_trait]
/// impl MigratorTrait for M20210101020202DoAThing {
///     async fn up(&self, mg: &MigrationManager) -> Result<(), DbErr> {
///         println!("up: M20210101020202DoAThing");
///         Ok(())
///     }
///     async fn down(&self, mg: &MigrationManager) -> Result<(), DbErr> {
///         println!("down: M20210101020202DoAThing");
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait MigratorTrait: MigrationName {
    /// up is run to apply a database migration. You can assume anything created in here doesn't exist when it is run. If an error occurs the `down` method will be run to undo the migration before retrying.
    async fn up(&self, mg: &MigrationManager) -> Result<(), DbErr>;

    /// down is used to undo a database migration. You should assume that anything applied in the `up` function is not necessarily created when this is run as the `up` function may have failed.
    async fn down(&self, mg: &MigrationManager) -> Result<(), DbErr>;
}

/// MigrationManager is used to manage migrations. It holds the database connection and has many helpers to make your database migration code concise.
pub struct MigrationManager<'a> {
    /// db holds the database connection. This can be used to run any custom queries again the database.
    pub db: &'a DbConn,
}

impl<'a> MigrationManager<'a> {
    /// new will create a new MigrationManager. This is primarily designed for internal use but is exposed in case you want to use it.
    pub fn new(db: &'a DbConn) -> Self {
        Self { db }
    }

    /// create_table will create a database table if it does not exist for a SeaORM Entity.
    ///
    /// ```rust
    /// use sea_orm::{Database, DbErr};
    /// use sea_orm::entity::prelude::*;
    /// use sea_migrations::MigrationManager;
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
    /// async fn main() -> Result<(), DbErr> {
    ///     let db = Database::connect("sqlite::memory:").await?;
    ///     // You would not normally create a MigrationManager by yourself. It would be provided to the `up` or `down` function by sea_migrations.
    ///     let mg = MigrationManager::new(&db);
    ///
    ///     mg.create_table(crate::Entity).await?; // Replace "crate" with the name of the module containing your SeaORM Model.
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn create_table<E: 'static>(&self, entity: E) -> Result<ExecResult, DbErr>
    where
        E: EntityTrait,
    {
        let mut stmt = Table::create();
        stmt.table(entity).if_not_exists();

        for column in E::Column::iter() {
            stmt.col(&mut get_column_def::<E>(column));
        }

        for relation in E::Relation::iter() {
            if relation.def().is_owner {
                continue;
            }
            stmt.foreign_key(&mut get_column_foreign_key_def::<E>(relation));
        }

        self.db
            .execute(self.db.get_database_backend().build(&stmt))
            .await
    }

    /// drop_table will drop a database table and all of it's data for a SeaORM Entity.
    ///
    /// ```rust
    /// use sea_orm::{Database, DbErr};
    /// use sea_orm::entity::prelude::*;
    /// use sea_migrations::MigrationManager;
    ///
    /// #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    /// #[sea_orm(table_name = "cake")]
    /// pub struct Model {
    ///     #[sea_orm(primary_key)]
    ///     pub id: i32,
    ///     pub name: String,
    /// }
    ///
    /// #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    /// pub enum Relation {}
    ///
    /// impl ActiveModelBehavior for ActiveModel {}
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), DbErr> {
    ///     let db = Database::connect("sqlite::memory:").await?;
    ///     // You would not normally create a MigrationManager by yourself. It would be provided to the `up` or `down` function by sea_migrations.
    ///     let mg = MigrationManager::new(&db);
    ///
    ///     mg.drop_table(crate::Entity).await?; // Replace "crate" with the name of the module containing your SeaORM Model.
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn drop_table<E: 'static>(&self, entity: E) -> Result<ExecResult, DbErr>
    where
        E: EntityTrait,
    {
        let stmt = Table::drop().table(entity).if_exists().to_owned();
        self.db
            .execute(self.db.get_database_backend().build(&stmt))
            .await
    }

    /// add_column will automatically create a new column in the existing database table for a specific column on the Entity.
    ///
    /// ```rust
    /// use sea_orm::{Database, DbErr};
    /// use sea_migrations::MigrationManager;
    ///
    /// mod original_model {
    ///      use sea_orm::entity::prelude::*;
    ///     
    ///     #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    ///     #[sea_orm(table_name = "cake")]
    ///     pub struct Model {
    ///         #[sea_orm(primary_key)]
    ///         pub id: i32,
    ///         pub name: String,
    ///     }
    ///
    ///     #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    ///     pub enum Relation {}
    ///
    ///     impl ActiveModelBehavior for ActiveModel {}
    /// }
    ///
    /// mod updated_model {
    ///      use sea_orm::entity::prelude::*;
    ///     
    ///     #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    ///     #[sea_orm(table_name = "cake")]
    ///     pub struct Model {
    ///         #[sea_orm(primary_key)]
    ///         pub id: i32,
    ///         pub name: String,
    ///         pub new_column: String,
    ///     }
    ///
    ///     #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    ///     pub enum Relation {}
    ///
    ///     impl ActiveModelBehavior for ActiveModel {}
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), DbErr> {
    ///     let db = Database::connect("sqlite::memory:").await?;
    ///     // You would not normally create a MigrationManager by yourself. It would be provided to the `up` or `down` function by sea_migrations.
    ///     let mg = MigrationManager::new(&db);
    ///     mg.create_table(original_model::Entity).await?; // Create the original table without the new column. This would have been done in the previous version of your application.
    ///
    ///     mg.add_column(updated_model::Entity, updated_model::Column::NewColumn).await?; // Replace "updated_model" with the name of the module containing your SeaORM Model and NewColumn with the name of your new Column.
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn add_column<E: 'static, T: 'static>(
        &self,
        entity: E,
        column: T,
    ) -> Result<ExecResult, DbErr>
    where
        E: EntityTrait<Column = T>,
        T: ColumnTrait,
    {
        let mut stmt = Table::alter();
        stmt.table(entity)
            .add_column(&mut get_column_def::<E>(column));

        self.db
            .execute(self.db.get_database_backend().build(&stmt))
            .await
    }

    /// drop_column will drop a table's column and all of it's data for a Column on a SeaORM Entity.
    ///
    /// The example panics due to SQLite not being able to drop a column.
    /// ```should_panic
    /// use sea_orm::{Database, DbErr};
    /// use sea_orm::entity::prelude::*;
    /// use sea_migrations::MigrationManager;
    ///
    /// #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    /// #[sea_orm(table_name = "cake")]
    /// pub struct Model {
    ///     #[sea_orm(primary_key)]
    ///     pub id: i32,
    ///     pub name: String,
    ///     pub column_to_remove: String, // Note: This column although removed from the database can NEVER be removed from the Model without causing issues with running older migrations.
    /// }
    ///
    /// #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    /// pub enum Relation {}
    ///
    /// impl ActiveModelBehavior for ActiveModel {}
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), DbErr> {
    ///     let db = Database::connect("sqlite::memory:").await?;
    ///     // You would not normally create a MigrationManager by yourself. It would be provided to the `up` or `down` function by sea_migrations.
    ///     let mg = MigrationManager::new(&db);
    ///     mg.create_table(crate::Entity).await?; // Create the original table with the column. This would have been done in the previous version of your application.
    ///
    ///     mg.drop_column(crate::Entity, crate::Column::ColumnToRemove).await?; // Replace "crate" with the name of the module containing your SeaORM Model and ColumnToRemove with the name of the column to remove.
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn drop_column<E: 'static, T: 'static>(
        &self,
        entity: E,
        column: T,
    ) -> Result<ExecResult, DbErr>
    where
        E: EntityTrait<Column = T>,
        T: ColumnTrait,
    {
        let mut stmt = Table::alter();
        stmt.table(entity).drop_column(column);

        self.db
            .execute(self.db.get_database_backend().build(&stmt))
            .await
    }
}

/// Migrator is used to handle running migration operations.
pub struct Migrator;

impl Migrator {
    /// run will run all of the database migrations provided via the migrations parameter.
    /// In microservice environments think about how this function is called. It contains an internal lock to prevent multiple clients running migrations at the same time but don't rely on it!
    ///
    /// ```rust
    /// use sea_migrations::Migrator;
    /// use sea_orm::{ Database, DbErr };
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), DbErr> {
    ///     let db = Database::connect("sqlite::memory:").await?;
    ///     
    ///     Migrator::run(
    ///         &db,
    ///         &mut vec![
    ///            // Box::new(models::M20210101020202DoAThing),
    ///         ],
    ///     )
    ///     .await
    /// }
    ///
    /// ```
    // Note(oscar): I don't like that the migrations argument is mutable but it works for now and that argument will be removed in a future version so their is no point trying to fix it.
    pub async fn run(
        db: &DbConn,
        migrations: &mut Vec<Box<dyn MigratorTrait>>,
    ) -> Result<(), DbErr> {
        let mg = MigrationManager::new(db);
        migrations_table::init(db).await?;
        migrations_table::lock(db).await?;
        let result = Self::do_migrations(&mg, migrations).await;
        migrations_table::unlock(db).await?;
        result
    }

    // do_migrations runs the Database migrations. This function exists so it is easier to capture the error in the `run` function.
    async fn do_migrations<'a>(
        mg: &'a MigrationManager<'a>,
        migrations: &mut Vec<Box<dyn MigratorTrait>>,
    ) -> Result<(), DbErr> {
        // Sort migrations into predictable order
        migrations.sort_by(|a, b| a.name().cmp(b.name()));

        for migration in migrations.iter() {
            let migration_name = migration.name().to_string();
            let migration_entry = migrations_table::get_version(mg.db, &migration_name).await?;

            match migration_entry {
                Some(_) => {}
                None => {
                    let result = migration.up(mg).await;
                    match result {
                        Ok(_) => {
                            migrations_table::insert_migration(mg.db, &migration_name).await?;
                        }
                        Err(err) => {
                            migration.down(mg).await?;
                            return Err(err);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
