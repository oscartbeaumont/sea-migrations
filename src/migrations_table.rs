use sea_orm::{
    sea_query::{Alias, ColumnDef, Expr, Query, Table},
    ConnectionTrait, DbConn, DbErr, QueryResult, Value,
};

// MIGRATIONS_TABLE_NAME is the name of the table created in the Database to keep track of the current state of the migrations.
const MIGRATIONS_TABLE_NAME: &str = "_sea_migrations";

// MIGRATIONS_TABLE_VERSION_COLUMN is the name of the column used to store the version of the migrations within the table used to track to current state of migrations.
const MIGRATIONS_TABLE_VERSION_COLUMN: &str = "version";

// MIGRATIONS_TABLE_LOCK_ROW_VERSION is the version contained in the row that is used to lock the table. If it exists then the table is locked and migrations are in progress. This should prevent any other process from running migrations at the same time.
const MIGRATIONS_TABLE_LOCK_ROW_VERSION: &str = "_lock";

/// init will create the migrations table in the database if it does not exist.
pub async fn init(db: &DbConn) -> Result<(), DbErr> {
    let stmt = Table::create()
        .table(Alias::new(MIGRATIONS_TABLE_NAME))
        .if_not_exists()
        .col(
            ColumnDef::new(Alias::new(MIGRATIONS_TABLE_VERSION_COLUMN))
                .string()
                .not_null()
                .primary_key(),
        )
        .to_owned();

    db.execute(db.get_database_backend().build(&stmt)).await?;
    Ok(())
}

/// lock will mark the migrations table as locked. This should prevent any other process from running migrations at the same time.
pub async fn lock(db: &DbConn) -> Result<(), DbErr> {
    // Check table lock
    let stmt = Query::select()
        .column(Alias::new(MIGRATIONS_TABLE_VERSION_COLUMN))
        .and_where(
            Expr::col(Alias::new(MIGRATIONS_TABLE_VERSION_COLUMN)).eq(Value::String(Some(
                Box::new(MIGRATIONS_TABLE_LOCK_ROW_VERSION.to_string()),
            ))),
        )
        .from(Alias::new(MIGRATIONS_TABLE_NAME))
        .to_owned();

    let result = db.query_one(db.get_database_backend().build(&stmt)).await?;
    if result.is_some() {
        return Err(DbErr::Custom(
            "Migrations table is locked! Please try again later!".into(),
        ));
    }

    // Create table lock
    let stmt = Query::insert()
        .into_table(Alias::new(MIGRATIONS_TABLE_NAME))
        .columns(vec![Alias::new(MIGRATIONS_TABLE_VERSION_COLUMN)])
        .values_panic(vec![Value::String(Some(Box::new(
            MIGRATIONS_TABLE_LOCK_ROW_VERSION.to_string(),
        )))])
        .to_owned();

    db.execute(db.get_database_backend().build(&stmt)).await?;
    Ok(())
}

/// unlock will unmark the migrations table as locked. This will allow any other process to run migrations.
pub async fn unlock(db: &DbConn) -> Result<(), DbErr> {
    let stmt = Query::delete()
        .from_table(Alias::new(MIGRATIONS_TABLE_NAME))
        .and_where(
            Expr::col(Alias::new(MIGRATIONS_TABLE_VERSION_COLUMN)).eq(Value::String(Some(
                Box::new(MIGRATIONS_TABLE_LOCK_ROW_VERSION.to_string()),
            ))),
        )
        .to_owned();

    db.execute(db.get_database_backend().build(&stmt)).await?;
    Ok(())
}

/// get_version will return a migration event with a given name from the database.
pub async fn get_version(db: &DbConn, version: String) -> Result<Option<QueryResult>, DbErr> {
    let stmt = Query::select()
        .column(Alias::new(MIGRATIONS_TABLE_VERSION_COLUMN))
        .and_where(
            Expr::col(Alias::new(MIGRATIONS_TABLE_VERSION_COLUMN))
                .eq(Value::String(Some(Box::new(version)))),
        )
        .from(Alias::new(MIGRATIONS_TABLE_NAME))
        .to_owned();

    db.query_one(db.get_database_backend().build(&stmt)).await
}

/// insert_migration will create a new migration event in the database.
pub async fn insert_migration(db: &DbConn, version: String) -> Result<u32, DbErr> {
    let stmt = Query::insert()
        .into_table(Alias::new(MIGRATIONS_TABLE_NAME))
        .columns(vec![Alias::new(MIGRATIONS_TABLE_VERSION_COLUMN)])
        .values_panic(vec![Value::String(Some(Box::new(version)))])
        .to_owned();

    let result = db.execute(db.get_database_backend().build(&stmt)).await?;
    Ok(result.last_insert_id() as u32)
}
