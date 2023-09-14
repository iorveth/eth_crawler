use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "transactions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub tx_id: String,
    #[sea_orm(column_type = "Unsigned")]
    pub block_number: u64,
    #[sea_orm(column_type = "Date")]
    pub date_time: DateTime,
    pub address_from: String,
    pub address_to: String,
    #[sea_orm(column_type = "Unsigned")]
    pub value: u64,
    #[sea_orm(column_type = "Unsigned")]
    pub tx_fee: u64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
