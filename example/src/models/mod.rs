use async_trait::async_trait;
use sea_migrations::{Migration, MigrationManager, MigratorTrait};
use sea_orm::DbErr;

pub mod customer;
pub mod customer2;
pub mod tax_info; // Customer -> Tax Info (1:1)

#[derive(Migration)]
pub struct M20210101020202DoAThing;

#[async_trait]
impl MigratorTrait for M20210101020202DoAThing {
    async fn up(&self, mg: &MigrationManager) -> Result<(), DbErr> {
        println!("up: M20210101020202DoAThing");
        mg.create_table(customer::Entity).await?;
        mg.create_table(tax_info::Entity).await?;
        Ok(())
    }
    async fn down(&self, mg: &MigrationManager) -> Result<(), DbErr> {
        println!("down: M20210101020202DoAThing");
        mg.drop_table(customer::Entity).await?;
        mg.drop_table(tax_info::Entity).await?;
        Ok(())
    }
}

#[derive(Migration)]
pub struct M20210105020202DoAThingAgain;

#[async_trait]
impl MigratorTrait for M20210105020202DoAThingAgain {
    async fn up(&self, mg: &MigrationManager) -> Result<(), DbErr> {
        println!("up: M20210105020202DoAThingAgain");
        mg.add_column(customer2::Entity, customer2::Column::SomeValue)
            .await?;

        // If you need to do anything special you have the full power of sea_query by using the DB instance at `mg.db`

        Ok(())
    }
    async fn down(&self, mg: &MigrationManager) -> Result<(), DbErr> {
        println!("down: M20210105020202DoAThingAgain");
        mg.drop_column(customer2::Entity, customer2::Column::SomeValue)
            .await?;
        Ok(())
    }
}
