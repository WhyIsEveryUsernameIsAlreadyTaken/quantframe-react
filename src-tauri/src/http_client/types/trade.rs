use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PlayerTrade {
    #[serde(rename = "user_name")]
    pub user_name: String,
}