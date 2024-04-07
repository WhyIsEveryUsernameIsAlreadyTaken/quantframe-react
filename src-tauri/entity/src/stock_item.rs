//! SeaORM Entity. Generated by sea-orm-codegen 0.3.2

use sea_orm::{entity::prelude::*, FromJsonQueryResult};
use serde::{Deserialize, Serialize};

use crate::{enums::stock_status::StockStatus, price_history::PriceHistory, sub_type::SubType};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "stock_item")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i64,
    pub wfm_id: String,
    pub wfm_url: String,
    pub item_name: String,
    pub item_unique_name: String,
    pub sub_type: Option<SubType>,
    pub bought: i64,
    pub minimum_price: Option<i64>,
    pub list_price: Option<i64>,
    pub owned: i64,
    pub is_hidden: bool,
    pub status: StockStatus,
    #[sea_orm(column_type = "Text")]
    pub price_history: PriceHistoryVec,
    pub updated_at: DateTimeUtc,
    #[sea_orm(created_at)]
    pub created_at: DateTimeUtc,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct PriceHistoryVec(pub Vec<PriceHistory>);

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}