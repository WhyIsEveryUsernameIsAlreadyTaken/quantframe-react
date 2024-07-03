use std::path::PathBuf;

use eyre::eyre;

use crate::{
    cache::{client::CacheClient, types::cache_skin::CacheSkin},
    helper,
    utils::modules::error::AppError,
};

#[derive(Clone, Debug)]
pub struct SkinModule {
    pub client: CacheClient,
    // debug_id: String,
    component: String,
    path: PathBuf,
    pub items: Vec<CacheSkin>,
}

impl SkinModule {
    pub fn new(client: CacheClient) -> Self {
        SkinModule {
            client,
            // debug_id: "ch_client_auction".to_string(),
            component: "Skin".to_string(),
            path: PathBuf::from("item/Skins.json"),
            items: Vec::new(),
        }
    }
    fn get_component(&self, component: &str) -> String {
        format!("{}:{}", self.component, component)
    }
    fn update_state(&self) {
        self.client.update_skin_module(self.clone());
    }

    pub fn load(&mut self) -> Result<(), AppError> {
        let content = self.client.read_text_from_file(&self.path)?;
        let items: Vec<CacheSkin> = serde_json::from_str(&content).map_err(|e| {
            AppError::new(
                self.get_component("Load").as_str(),
                eyre!(format!("Failed to parse SkinModule from file: {}", e)),
            )
        })?;
        self.items = items;
        self.update_state();
        Ok(())
    }
    pub fn get_by(&self, input: &str, by: &str) -> Result<Option<CacheSkin>, AppError> {
        let items = self.items.clone();
        let args = match helper::validate_args(by, vec!["--item_by"]) {
            Ok(args) => args,
            Err(e) => return Err(e),
        };
        let mode = args.get("--item_by").unwrap();
        let case_insensitive = args.get("--case_insensitive").is_some();
        // let lang = args.get("--item_lang").unwrap_or(&"en".to_string());
        let remove_string = args.get("--remove_string");

        let item = if mode == "name" {
            items
                .iter()
                .find(|x| helper::create_key(&x.name, case_insensitive, remove_string) == input)
                .cloned()
        } else if mode == "unique_name" {
            items
                .iter()
                .find(|x| {
                    helper::create_key(&x.unique_name, case_insensitive, remove_string) == input
                })
                .cloned()
        } else {
            return Err(AppError::new(
                &self.get_component("GetBy"),
                eyre!("Invalid by value: {}", by),
            ));
        };
        Ok(item)
    }
}
