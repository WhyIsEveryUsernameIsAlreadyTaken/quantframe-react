use ::entity::{stock_item, stock_item::Entity as StockItem};

use sea_orm::{sea_query::Expr, *};

pub struct StockItemQuery;

impl StockItemQuery {
    pub async fn find_all_transactions(db: &DbConn) -> Result<Vec<stock_item::Model>, DbErr> {
        StockItem::find().all(db).await
    }

    pub async fn get_all(db: &DbConn) -> Result<Vec<stock_item::Model>, DbErr> {
        StockItem::find().all(db).await
    }

    pub async fn get_all_stock_items(
        db: &DbConn,
        minimum_owned: i32,
    ) -> Result<Vec<stock_item::Model>, DbErr> {
        StockItem::find()
            .filter(Expr::col(stock_item::Column::Owned).gt(minimum_owned))
            .all(db)
            .await
    }
}