use std::{
    fs::File,
    io::Write,
    sync::{Arc, Mutex},
};

use entity::{
    enums::stock_status::StockStatus,
    price_history::PriceHistoryVec,
    stock_riven::{self, MatchRivenStruct, RivenAttribute, RivenAttributeVec},
    sub_type::SubType,
    transaction::TransactionItemType,
};
use eyre::eyre;
use serde_json::{json, Value};
use service::{StockItemMutation, StockRivenMutation, TransactionMutation};

use crate::{
    app::client::AppState,
    cache::client::CacheClient,
    notification::client::NotifyClient,
    utils::{
        enums::ui_events::{UIEvent, UIOperationEvent},
        modules::{error::AppError, logger},
    },
    wfm_client::{client::WFMClient, enums::order_type::OrderType, types::order_by_item},
};

#[tauri::command]
pub async fn stock_riven_create(
    wfm_url: String,
    bought: i64,
    mod_name: String,
    mastery_rank: i64,
    re_rolls: i64,
    polarity: String,
    rank: i64,
    attributes: Vec<RivenAttribute>,
    minimum_price: Option<i64>,
    is_hidden: Option<bool>,
    app: tauri::State<'_, Arc<Mutex<AppState>>>,
    cache: tauri::State<'_, Arc<Mutex<CacheClient>>>,
    notify: tauri::State<'_, Arc<Mutex<NotifyClient>>>,
) -> Result<stock_riven::Model, AppError> {
    let app = app.lock()?.clone();
    let cache = cache.lock()?.clone();
    let notify = notify.lock()?.clone();

    // Check if the weapon is exist in the cache.
    let weapon = match cache.riven().find_riven_type_by_url_name(&wfm_url) {
        Some(weapon) => weapon,
        None => {
            return Err(AppError::new(
                "StockRivenCreate",
                eyre!(format!("Weapon not found: {}", wfm_url)),
            ))
        }
    };

    // Validate the attributes
    for attribute in attributes.iter() {
        match cache
            .riven()
            .find_riven_attribute_by_url_name(&attribute.url_name)
        {
            Some(_) => {}
            None => {
                return Err(AppError::new(
                    "StockRivenCreate",
                    eyre!(format!("Invalid attribute: {:?}", attribute)),
                ))
            }
        }
    }

    // Create the stock item
    let stock = entity::stock_riven::Model::new(
        weapon.wfm_id.clone(),
        wfm_url.clone(),
        None,
        weapon.i18_n["en"].name.clone(),
        weapon.riven_type.clone(),
        weapon.unique_name.clone(),
        rank,
        mod_name,
        RivenAttributeVec(attributes),
        mastery_rank,
        re_rolls,
        polarity,
        bought,
        minimum_price,
        is_hidden.unwrap_or(true),
        "".to_string(),
    );
    match StockRivenMutation::create(&app.conn, stock.clone()).await {
        Ok(stock) => {
            notify.gui().send_event_update(
                UIEvent::UpdateStockRivens,
                UIOperationEvent::CreateOrUpdate,
                Some(json!(stock)),
            );
        }
        Err(e) => return Err(AppError::new("StockRivenCreate", eyre!(e))),
    }
    if bought == 0 {
        return Ok(stock);
    }
    // Add Transaction to the database
    let transaction = entity::transaction::Model::new(
        stock.wfm_weapon_id.clone(),
        stock.wfm_weapon_url.clone(),
        stock.weapon_name.clone(),
        TransactionItemType::Riven,
        stock.weapon_unique_name.clone(),
        stock.sub_type.clone(),
        vec![stock.weapon_type.clone()],
        entity::transaction::TransactionType::Purchase,
        1,
        "".to_string(),
        bought,
        Some(json!({
            "mod_name": stock.mod_name,
            "mastery_rank": stock.mastery_rank,
            "re_rolls": stock.re_rolls,
            "polarity": stock.polarity,
            "attributes": stock.attributes,
        })),
    );

    match TransactionMutation::create(&app.conn, transaction).await {
        Ok(inserted) => {
            notify.gui().send_event_update(
                UIEvent::UpdateTransaction,
                UIOperationEvent::CreateOrUpdate,
                Some(json!(inserted)),
            );
        }
        Err(e) => return Err(AppError::new("StockItemCreate", eyre!(e))),
    }
    Ok(stock)
}

#[tauri::command]
pub async fn stock_riven_update(
    id: i64,
    minimum_price: Option<i64>,
    sub_type: Option<SubType>,
    is_hidden: Option<bool>,
    filter: Option<MatchRivenStruct>,
    app: tauri::State<'_, Arc<Mutex<AppState>>>,
    notify: tauri::State<'_, Arc<Mutex<NotifyClient>>>,
) -> Result<entity::stock_riven::Model, AppError> {
    let app = app.lock()?.clone();
    let notify = notify.lock()?.clone();

    let stock = match StockRivenMutation::find_by_id(&app.conn, id).await {
        Ok(stock) => stock,
        Err(e) => return Err(AppError::new("StockRivenUpdate", eyre!(e))),
    };

    if stock.is_none() {
        return Err(AppError::new(
            "StockRivenUpdate",
            eyre!(format!("Stock Riven not found: {}", id)),
        ));
    }

    let mut stock = stock.unwrap();

    if let Some(minimum_price) = minimum_price {
        stock.minimum_price = Some(minimum_price);
    }

    if let Some(sub_type) = sub_type {
        stock.sub_type = Some(sub_type);
    }

    if let Some(filter) = filter {
        stock.filter = filter;
    }

    if let Some(is_hidden) = is_hidden {
        stock.is_hidden = is_hidden;
    }
    stock.updated_at = chrono::Utc::now();

    match StockRivenMutation::update_by_id(&app.conn, stock.id, stock.clone()).await {
        Ok(updated) => {
            notify.gui().send_event_update(
                UIEvent::UpdateStockRivens,
                UIOperationEvent::CreateOrUpdate,
                Some(json!(updated)),
            );
        }
        Err(e) => return Err(AppError::new("StockItemUpdate", eyre!(e))),
    }

    Ok(stock)
}

#[tauri::command]
pub async fn stock_riven_update_bulk(
    ids: Vec<i64>,
    minimum_price: Option<i64>,
    is_hidden: Option<bool>,
    app: tauri::State<'_, Arc<Mutex<AppState>>>,
    notify: tauri::State<'_, Arc<Mutex<NotifyClient>>>,
) -> Result<i64, AppError> {
    let app = app.lock()?.clone();
    let notify = notify.lock()?.clone();
    let mut total: i64 = 0;
    for id in ids {
        let stock = match StockRivenMutation::find_by_id(&app.conn, id).await {
            Ok(stock) => stock,
            Err(e) => return Err(AppError::new("StockRivenUpdate", eyre!(e))),
        };

        if stock.is_none() {
            return Err(AppError::new(
                "StockRivenUpdate",
                eyre!(format!("Stock Riven not found: {}", id)),
            ));
        }
        total += 1;
        let mut stock = stock.unwrap();

        if let Some(minimum_price) = minimum_price {
            stock.minimum_price = Some(minimum_price);
        }

        if let Some(is_hidden) = is_hidden {
            stock.is_hidden = is_hidden;
        }
        stock.updated_at = chrono::Utc::now();

        match StockRivenMutation::update_by_id(&app.conn, stock.id, stock.clone()).await {
            Ok(updated) => {
                notify.gui().send_event_update(
                    UIEvent::UpdateStockRivens,
                    UIOperationEvent::CreateOrUpdate,
                    Some(json!(updated)),
                );
            }
            Err(e) => return Err(AppError::new("StockItemUpdate", eyre!(e))),
        }
    }
    Ok(total)
}
#[tauri::command]
pub async fn stock_riven_delete_bulk(
    ids: Vec<i64>,
    app: tauri::State<'_, Arc<Mutex<AppState>>>,
    notify: tauri::State<'_, Arc<Mutex<NotifyClient>>>,
    wfm: tauri::State<'_, Arc<Mutex<WFMClient>>>,
) -> Result<i64, AppError> {
    let mut total: i64 = 0;
    for id in ids {
        match stock_riven_delete(id, app.clone(), notify.clone(), wfm.clone()).await {
            Ok(_) => {
                total += 1;
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    Ok(total)
}

#[tauri::command]
pub async fn stock_riven_sell(
    id: i64,
    price: i64,
    app: tauri::State<'_, Arc<Mutex<AppState>>>,
    notify: tauri::State<'_, Arc<Mutex<NotifyClient>>>,
    wfm: tauri::State<'_, Arc<Mutex<WFMClient>>>,
) -> Result<entity::stock_riven::Model, AppError> {
    let app = app.lock()?.clone();
    let notify = notify.lock()?.clone();
    let wfm = wfm.lock()?.clone();
    let stock = match StockRivenMutation::find_by_id(&app.conn, id).await {
        Ok(stock) => stock,
        Err(e) => return Err(AppError::new("StockRivenSell", eyre!(e))),
    };

    if stock.is_none() {
        return Err(AppError::new(
            "StockRivenSell",
            eyre!(format!("Stock Riven not found: {}", id)),
        ));
    }
    let stock = stock.unwrap();

    // Delete the auction from WFM
    if stock.wfm_order_id.is_some() {
        match wfm
            .auction()
            .delete(&stock.wfm_order_id.clone().unwrap())
            .await
        {
            Ok(auction) => {
                if auction.is_some() {
                    // Send Update to the UI
                }
            }
            Err(e) => {
                if e.cause().contains("app.form.not_exist") {
                    logger::info_con(
                        "StockRivenSell",
                        format!("Error deleting auction: {}", e.cause()).as_str(),
                    );
                } else {
                    return Err(e);
                }
            }
        }
    }

    // Add Transaction to the database
    let transaction = entity::transaction::Model::new(
        stock.wfm_weapon_id.clone(),
        stock.wfm_weapon_url.clone(),
        stock.weapon_name.clone(),
        TransactionItemType::Item,
        stock.weapon_unique_name.clone(),
        stock.sub_type.clone(),
        vec![stock.weapon_type.clone()],
        entity::transaction::TransactionType::Sale,
        1,
        "".to_string(),
        price,
        None,
    );

    match TransactionMutation::create(&app.conn, transaction).await {
        Ok(inserted) => {
            notify.gui().send_event_update(
                UIEvent::UpdateTransaction,
                UIOperationEvent::CreateOrUpdate,
                Some(json!(inserted)),
            );
        }
        Err(e) => return Err(AppError::new("StockItemSell", eyre!(e))),
    }

    // Delete the stock from the database
    match StockRivenMutation::delete(&app.conn, stock.id).await {
        Ok(_) => {
            notify.gui().send_event_update(
                UIEvent::UpdateStockRivens,
                UIOperationEvent::Delete,
                Some(json!({ "id": stock.id })),
            );
        }
        Err(e) => return Err(AppError::new("StockItemSell", eyre!(e))),
    }

    Ok(stock)
}

#[tauri::command]
pub async fn stock_riven_delete(
    id: i64,
    app: tauri::State<'_, Arc<Mutex<AppState>>>,
    notify: tauri::State<'_, Arc<Mutex<NotifyClient>>>,
    wfm: tauri::State<'_, Arc<Mutex<WFMClient>>>,
) -> Result<(), AppError> {
    let app = app.lock()?.clone();
    let notify = notify.lock()?.clone();
    let wfm = wfm.lock()?.clone();

    let stock_item = match StockRivenMutation::find_by_id(&app.conn, id).await {
        Ok(stock) => stock,
        Err(e) => return Err(AppError::new("StockRivenDelete", eyre!(e))),
    };

    if stock_item.is_none() {
        return Err(AppError::new(
            "StockRivenDelete",
            eyre!(format!("Stock Riven not found: {}", id)),
        ));
    }
    let stock_item = stock_item.unwrap();

    // Delete the auction from WFM
    if stock_item.wfm_order_id.is_some() {
        match wfm
            .auction()
            .delete(&stock_item.wfm_order_id.clone().unwrap())
            .await
        {
            Ok(auction) => {
                if auction.is_some() {
                    notify.gui().send_event_update(
                        UIEvent::UpdateAuction,
                        UIOperationEvent::Delete,
                        Some(json!({ "id": id })),
                    );
                }
            }
            Err(e) => {
                if e.cause().contains("app.form.not_exist") {
                    logger::info_con(
                        "StockRivenSell",
                        format!("Error deleting auction: {}", e.cause()).as_str(),
                    );
                } else {
                    return Err(e);
                }
            }
        }
    }
    match StockRivenMutation::delete(&app.conn, stock_item.id).await {
        Ok(deleted) => {
            if deleted.rows_affected > 0 {
                notify.gui().send_event_update(
                    UIEvent::UpdateStockRivens,
                    UIOperationEvent::Delete,
                    Some(json!({ "id": id })),
                );
            }
        }
        Err(e) => return Err(AppError::new("StockRivenDelete", eyre!(e))),
    }
    Ok(())
}
