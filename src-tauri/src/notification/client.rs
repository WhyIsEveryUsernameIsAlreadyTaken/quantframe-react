use std::sync::{Arc, RwLock};

use tauri::AppHandle;

use crate::utils::modules::error::AppError;

use super::modules::{discord::DiscordModule, gui::GUIModule, system::SystemModule};

#[derive(Clone, Debug)]
pub struct NotifyClient {
    pub log_file: String,
    pub app_handler: AppHandle,
    pub component: String,
    // Modules will be added here
    pub system_module: Arc<RwLock<Option<SystemModule>>>,
    pub gui_module: Arc<RwLock<Option<GUIModule>>>,
    pub discord_module: Arc<RwLock<Option<DiscordModule>>>,
}

impl NotifyClient {
    pub fn new(app_handler: AppHandle) -> Self {
        NotifyClient {      
            app_handler,      
            log_file: "notify.log".to_string(),
            component: "NotifyClient".to_string(),
            system_module: Arc::new(RwLock::new(None)),
            gui_module: Arc::new(RwLock::new(None)),
            discord_module: Arc::new(RwLock::new(None)),
        }
    }

    pub fn system(&self) -> SystemModule {
        // Lazily initialize SystemModule if not already initialized
        if self.system_module.read().unwrap().is_none() {
            *self.system_module.write().unwrap() = Some(SystemModule::new(self.clone()).clone());
        }

        // Unwrapping is safe here because we ensured the item_module is initialized
        self.system_module.read().unwrap().as_ref().unwrap().clone()
    }
    pub fn update_system_module(&self, module: SystemModule) {
        // Update the stored SystemModule
        *self.system_module.write().unwrap() = Some(module);
    }

    pub fn gui(&self) -> GUIModule {
        // Lazily initialize ItemModule if not already initialized
        if self.gui_module.read().unwrap().is_none() {
            *self.gui_module.write().unwrap() = Some(GUIModule::new(self.clone()).clone());
        }

        // Unwrapping is safe here because we ensured the gui_module is initialized
        self.gui_module.read().unwrap().as_ref().unwrap().clone()
    }
    pub fn update_gui_module(&self, module: GUIModule) {
        // Update the stored GUIModule
        *self.gui_module.write().unwrap() = Some(module);
    }

    pub fn discord(&self) -> DiscordModule {
        // Lazily initialize DiscordModule if not already initialized
        if self.discord_module.read().unwrap().is_none() {
            *self.discord_module.write().unwrap() = Some(DiscordModule::new(self.clone()).clone());
        }

        // Unwrapping is safe here because we ensured the item_module is initialized
        self.discord_module
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone()
    }
    pub fn update_discord_module(&self, module: DiscordModule) {
        // Update the stored DiscordModule
        *self.discord_module.write().unwrap() = Some(module);
    }
}