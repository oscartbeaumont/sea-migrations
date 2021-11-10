use sea_orm::{
    sea_query::{Alias, ColumnDef, Expr, Query, Table},
    ConnectionTrait, DbConn, DbErr, Value,
};

// MIGRATIONS_TABLE_NAME is the name of the table created in the Database to keep track of the current state of the migrations.
const MIGRATIONS_TABLE_NAME: &str = "_sea_migrations";

// MIGRATIONS_TABLE_VERSION_COLUMN is the name of the column used to store the version of the migrations within the table used to track to current state of migrations.
const MIGRATIONS_TABLE_VERSION_COLUMN: &str = "version";

// MIGRATIONS_TABLE_LOCK_ROW_VERSION is the version number of a row that is created in the migrations table to mark the table as locked. This should prevent any other process from running migrations at the same time.
const MIGRATIONS_TABLE_LOCK_ROW_VERSION: i32 = -1;

/// init will create the migrations table in the database if it does not exist.
pub async fn init(db: &DbConn) -> Result<(), DbErr> {
    let stmt = Table::create()
        .table(Alias::new(MIGRATIONS_TABLE_NAME))
        .if_not_exists()
        .col(
            ColumnDef::new(Alias::new(MIGRATIONS_TABLE_VERSION_COLUMN))
                .integer()
                .not_null()
                .auto_increment()
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
            Expr::col(Alias::new(MIGRATIONS_TABLE_VERSION_COLUMN))
                .eq(Value::Int(Some(MIGRATIONS_TABLE_LOCK_ROW_VERSION))),
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
        .values_panic(vec![Value::Int(Some(-1))])
        .to_owned();

    db.execute(db.get_database_backend().build(&stmt)).await?;
    Ok(())
}

/// unlock will unmark the migrations table as locked. This will allow any other process to run migrations.
pub async fn unlock(db: &DbConn) -> Result<(), DbErr> {
    let stmt = Query::delete()
        .from_table(Alias::new(MIGRATIONS_TABLE_NAME))
        .and_where(
            Expr::col(Alias::new(MIGRATIONS_TABLE_VERSION_COLUMN))
                .eq(Value::Int(Some(MIGRATIONS_TABLE_LOCK_ROW_VERSION))),
        )
        .to_owned();

    db.execute(db.get_database_backend().build(&stmt)).await?;
    Ok(())
}

/// get_latest will return the version of the latest migration (or None if no migrations have previous been run).
pub async fn get_latest(db: &DbConn) -> Result<Option<u32>, DbErr> {
    let stmt = Query::select()
        .expr(Expr::cust(&format!(
            "MAX (`{}`) AS `{0}`",
            MIGRATIONS_TABLE_VERSION_COLUMN
        )))
        .and_where(
            Expr::col(Alias::new(MIGRATIONS_TABLE_VERSION_COLUMN))
                .ne(Value::Int(Some(MIGRATIONS_TABLE_LOCK_ROW_VERSION))),
        )
        .from(Alias::new(MIGRATIONS_TABLE_NAME))
        .to_owned();

    let result = db.query_one(db.get_database_backend().build(&stmt)).await?;
    if let Some(result) = result {
        let latest_migration_version = result.try_get("", MIGRATIONS_TABLE_VERSION_COLUMN);
        if let Ok(latest_migration_version) = latest_migration_version {
            Ok(Some(latest_migration_version))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

/// insert_migration will create a new migration event in the database.
pub async fn insert_migration(db: &DbConn) -> Result<u32, DbErr> {
    let stmt = Query::insert()
        .into_table(Alias::new(MIGRATIONS_TABLE_NAME))
        .columns(vec![Alias::new(MIGRATIONS_TABLE_VERSION_COLUMN)])
        .values_panic(vec![Value::Int(None)])
        .to_owned();

    let result = db.execute(db.get_database_backend().build(&stmt)).await?;
    Ok(result.last_insert_id() as u32)
}
