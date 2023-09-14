pub use sea_orm_migration::prelude::*;

mod m20230913_132440_create_transactions_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(
            m20230913_132440_create_transactions_table::Migration,
        )]
    }
}
