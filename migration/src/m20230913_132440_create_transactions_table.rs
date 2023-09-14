use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Transactions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Transactions::TxId)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Transactions::BlockNumber)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Transactions::DateTime)
                            .date_time()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Transactions::AddressTo).string().not_null())
                    .col(
                        ColumnDef::new(Transactions::AddressFrom)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Transactions::Value)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Transactions::TxFee)
                            .big_unsigned()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Transactions::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Transactions {
    Table,
    TxId,
    BlockNumber,
    DateTime,
    AddressTo,
    AddressFrom,
    Value,
    TxFee,
}
