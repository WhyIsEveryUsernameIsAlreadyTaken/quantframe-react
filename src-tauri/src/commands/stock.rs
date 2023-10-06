use std::sync::{Arc, Mutex};

use crate::{
    database::client::DBClient,
    error::{self, AppError},
    logger,
    structs::{Order, RivenAttribute},
    wfm_client::client::WFMClient,
};
use eyre::eyre;
use serde_json::json;

// Item Stock Commands
#[tauri::command]
pub async fn create_item_stock(
    id: String,
    report: bool,
    quantity: i32,
    price: f64,
    rank: i32,
    sub_type: Option<&str>,
    db: tauri::State<'_, Arc<Mutex<DBClient>>>,
    wfm: tauri::State<'_, Arc<Mutex<WFMClient>>>,
) -> Result<serde_json::Value, AppError> {
    let db = db.lock()?.clone();
    let wfm = wfm.lock()?.clone();

    match db
        .stock_item()
        .create(&id, quantity, price, rank, sub_type)
        .await
    {
        Ok(stockitem) => {
            // Create transaction
            match db
                .transaction()
                .create(&id, "item", "buy", quantity, price as i32, rank, None)
                .await
            {
                Ok(_) => {
                    // Send Close Event to Warframe Market API
                    if report {
                        wfm.orders().close(&id, "buy").await?;
                    }
                    return Ok(serde_json::to_value(stockitem).unwrap());
                }
                Err(e) => {
                    error::create_log_file(db.log_file.clone(), &e);
                    return Err(e);
                }
            };
        }
        Err(e) => {
            error::create_log_file(db.log_file.clone(), &e);
            return Err(e);
        }
    };
}

#[tauri::command]
pub async fn delete_item_stock(
    id: i64,
    db: tauri::State<'_, Arc<Mutex<DBClient>>>,
    wfm: tauri::State<'_, Arc<Mutex<WFMClient>>>,
) -> Result<serde_json::Value, AppError> {
    let db = db.lock()?.clone();
    let wfm = wfm.lock()?.clone();
    match db.stock_item().delete(id).await {
        Ok(stockitem) => {
            // Send Delete Event to Frontend
            db.stock_item()
                .emit("DELETE", serde_json::to_value(stockitem.clone()).unwrap());
            // Get all sell orders from Warframe Market API
            let ordres: Vec<Order> = wfm.orders().get_my_orders().await?.sell_orders;
            let order = ordres
                .iter()
                .find(|order| order.item.as_ref().unwrap().url_name == stockitem.url)
                .clone();
            // Delete order if it exists
            if order.is_some() {
                wfm.orders()
                    .delete(
                        &order.unwrap().id,
                        &stockitem.name,
                        &stockitem.wfm_id,
                        "sell",
                    )
                    .await?;
            }
            return Ok(serde_json::to_value(stockitem).unwrap());
        }
        Err(e) => {
            error::create_log_file(db.log_file.clone(), &e);
            return Err(e);
        }
    };
}

#[tauri::command]
pub async fn sell_item_stock(
    id: i64,
    report: bool,
    quantity: i32,
    price: i32,
    db: tauri::State<'_, Arc<Mutex<DBClient>>>,
    wfm: tauri::State<'_, Arc<Mutex<WFMClient>>>,
) -> Result<serde_json::Value, AppError> {
    let db = db.lock()?.clone();
    let wfm = wfm.lock()?.clone();
    match db.stock_item().sell_item(id, price, quantity).await {
        Ok(invantory) => {
            if invantory.owned == 0 {
                db.stock_item()
                    .emit("DELETE", serde_json::to_value(invantory.clone()).unwrap());
            } else {
                db.stock_item().emit(
                    "CREATE_OR_UPDATE",
                    serde_json::to_value(invantory.clone()).unwrap(),
                );
            }
            db.transaction()
                .create(
                    &invantory.url,
                    "item",
                    "sell",
                    quantity,
                    price,
                    invantory.rank,
                    None,
                )
                .await?;

            // Send Close Event to Warframe Market API
            if report {
                wfm.orders().close(&invantory.url, "sell").await?;
            } else {
                let ordres: Vec<Order> = wfm.orders().get_my_orders().await?.sell_orders;
                let order = ordres
                    .iter()
                    .find(|order| order.item.as_ref().unwrap().url_name == invantory.url)
                    .clone();
                if order.is_some() {
                    if invantory.owned <= 0 {
                        wfm.orders()
                            .delete(
                                &order.unwrap().id,
                                &invantory.name,
                                &invantory.wfm_id,
                                "sell",
                            )
                            .await?;
                    } else {
                        wfm.orders()
                            .update(
                                &order.unwrap().id,
                                order.unwrap().platinum as i32,
                                invantory.owned,
                                order.unwrap().visible,
                                &invantory.name,
                                &invantory.wfm_id,
                                "sell",
                            )
                            .await?;
                    }
                }
            }
            return Ok(serde_json::to_value(invantory).unwrap());
        }
        Err(e) => {
            error::create_log_file(db.log_file.clone(), &e);
            return Err(e);
        }
    };
}

// Riven Stock Commands
#[tauri::command]
pub async fn create_riven_stock(
    id: String,
    price: f64,
    rank: i32,
    attributes: Vec<RivenAttribute>,
    mastery_rank: i32,
    re_rolls: i32,
    polarity: &str,
    mod_name: &str,
    db: tauri::State<'_, Arc<Mutex<DBClient>>>,
    wfm: tauri::State<'_, Arc<Mutex<WFMClient>>>,
) -> Result<serde_json::Value, AppError> {
    let db = db.lock()?.clone();
    let riven_item = db
        .stock_riven()
        .create(
            None,
            &id,
            mod_name,
            price,
            rank,
            attributes,
            mastery_rank,
            re_rolls,
            polarity,
        )
        .await
        .map_err(|e| {
            error::create_log_file(db.log_file.clone(), &e);
            e
        })?;
    let item_value = serde_json::to_value(riven_item.clone()).unwrap();
    if riven_item.price <= 0.0 {
        return Ok(item_value);
    }
    db.transaction()
        .create(
            &riven_item.weapon_url,
            "riven",
            "buy",
            1,
            price as i32,
            riven_item.rank,
            Some(item_value.clone()),
        )
        .await
        .map_err(|e| {
            error::create_log_file(db.log_file.clone(), &e);
            e
        })?;

    Ok(item_value)
}
#[tauri::command]
pub async fn import_auction(
    id: String,
    price: i32,
    db: tauri::State<'_, Arc<Mutex<DBClient>>>,
    wfm: tauri::State<'_, Arc<Mutex<WFMClient>>>,
) -> Result<serde_json::Value, AppError> {
    let db = db.lock()?.clone();
    let wfm = wfm.lock()?.clone();
    let auctions = wfm.auction().get_my_auctions().await?;

    let auction = auctions.iter().find(|auction| auction.id == id).clone();
    if auction.is_none() {
        return Err(AppError::new(
            "Auction not found",
            eyre!("Auction not found"),
        ));
    }
    let auction = auction.unwrap().clone();

    let riven_item = db
        .stock_riven()
        .import_auction(auction.clone(), price)
        .await
        .map_err(|e| {
            error::create_log_file(db.log_file.clone(), &e);
            e
        })?;
    let item_value = serde_json::to_value(&auction.item).unwrap();
    if riven_item.price <= 0.0 {
        return Ok(item_value);
    }
    db.transaction()
        .create(
            &riven_item.weapon_url,
            "riven",
            "buy",
            1,
            price as i32,
            riven_item.rank,
            Some(item_value.clone()),
        )
        .await
        .map_err(|e| {
            error::create_log_file(db.log_file.clone(), &e);
            e
        })?;

    Ok(item_value)
}

#[tauri::command]
pub async fn delete_riven_stock(
    id: i32,
    db: tauri::State<'_, Arc<Mutex<DBClient>>>,
    wfm: tauri::State<'_, Arc<Mutex<WFMClient>>>,
) -> Result<serde_json::Value, AppError> {
    let db = db.lock()?.clone();
    let wfm = wfm.lock()?.clone();
    logger::warning_con(
        "CommandStock:",
        "Riven Stock Commands are not implemented yet",
    );
    Ok(json!({}))
}

#[tauri::command]
pub async fn sell_riven_stock(
    id: i32,
    report: bool,
    quantity: i32,
    price: f64,
    db: tauri::State<'_, Arc<Mutex<DBClient>>>,
    wfm: tauri::State<'_, Arc<Mutex<WFMClient>>>,
) -> Result<serde_json::Value, AppError> {
    let db = db.lock()?.clone();
    let wfm = wfm.lock()?.clone();
    logger::warning_con(
        "CommandStock:",
        "Riven Stock Commands are not implemented yet",
    );
    Ok(json!({}))
}

// -----------------------------------------------------------------------------------------------