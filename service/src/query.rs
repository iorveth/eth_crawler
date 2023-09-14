use ::entity::{transactions, transactions::Entity as Transactions};
use sea_orm::*;
use std::collections::BTreeSet;

pub struct Query;

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
enum QueryAs {
    BlockNumber,
}

impl Query {
    pub async fn get_transactions_count_since_block_for_selected_address(
        db: &DbConn,
        starting_block_id: u64,
        address: String,
    ) -> Result<u64, DbErr> {
        Transactions::find()
            .filter(
                Condition::any()
                    .add(transactions::Column::AddressFrom.eq(&address))
                    .add(transactions::Column::AddressTo.eq(&address)),
            )
            .filter(transactions::Column::BlockNumber.gt(starting_block_id))
            .count(db)
            .await
    }

    /// Retrieves transactions block numbers since starting block for selected address.
    pub async fn get_block_numbers_since_block_for_selected_address(
        db: &DbConn,
        address: &str,
        starting_block_id: u64,
    ) -> Result<BTreeSet<u64>, DbErr> {
        let last_transaction: Vec<u64> = Transactions::find()
            .filter(transactions::Column::BlockNumber.gte(starting_block_id))
            .filter(
                Condition::any()
                    .add(transactions::Column::AddressFrom.eq(address))
                    .add(transactions::Column::AddressTo.eq(address)),
            )
            .select_only()
            .column(transactions::Column::BlockNumber)
            .into_values::<_, QueryAs>()
            .all(db)
            .await?;

        Ok(last_transaction.into_iter().collect())
    }

    /// If ok, returns (post models, num pages).
    pub async fn find_transactions_in_page(
        db: &DbConn,
        address: String,
        starting_block_id: u64,
        page: u64,
        transactions_per_page: u64,
    ) -> Result<(Vec<transactions::Model>, u64), DbErr> {
        // Setup paginator
        let paginator = Transactions::find()
            .filter(transactions::Column::BlockNumber.gte(starting_block_id))
            .filter(
                Condition::any()
                    .add(transactions::Column::AddressFrom.eq(&address))
                    .add(transactions::Column::AddressTo.eq(&address)),
            )
            .order_by_asc(transactions::Column::TxId)
            .paginate(db, transactions_per_page);
        let num_pages = paginator.num_pages().await?;

        // Fetch paginated transactions
        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }
}
