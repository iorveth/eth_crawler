use ::entity::{transactions, transactions::Entity as Post};
use sea_orm::*;

pub struct Mutation;

impl Mutation {
    pub async fn save_transactions(
        db: &DbConn,
        unfetched_transactions: Vec<transactions::Model>,
    ) -> Result<(), DbErr> {
        let unfetched_transactions_models: Vec<_> = unfetched_transactions
            .into_iter()
            .map(|unfetched_transaction| transactions::ActiveModel {
                tx_id: Set(unfetched_transaction.tx_id.to_owned()),
                address_from: Set(unfetched_transaction.address_from.to_owned()),
                address_to: Set(unfetched_transaction.address_to.to_owned()),
                value: Set(unfetched_transaction.value.to_owned()),
                tx_fee: Set(unfetched_transaction.tx_fee.to_owned()),
                block_number: Set(unfetched_transaction.block_number),
                date_time: Set(unfetched_transaction.date_time),
            })
            .collect();

        Post::insert_many(unfetched_transactions_models)
            .exec(db)
            .await?;

        Ok(())
    }
}
