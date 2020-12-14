// Some notes on error handling in logger.rs Since this library is only used in the context
// of the app lxpservice, errors are not returned but are handled directly in the sense of
// the app. This simplifies the interface design to the library.

use log::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Profile {
    pub user_name: String,
    pub url: String,
    pub api_key: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct LxpConfig {
    app_name: String,
    profile_active: Option<String>,
    profiles: HashMap<String, Profile>,
}

impl LxpConfig {
    pub fn new(app_name: &str) -> LxpConfig {
        match confy::load::<LxpConfig>(app_name) {
            Err(_e) => {
                let lxp_config = LxpConfig {
                    app_name: app_name.into(),
                    profile_active: None,
                    profiles: HashMap::new(),
                };
                confy::store(app_name, &lxp_config).expect("Failed to write config file");
                error!("Could not read conifg file, new one created"); // exits app
                return lxp_config;
            }
            Ok(mut lxp_config) => {
                if lxp_config.app_name == "" {  // Set app_name on first run
                    lxp_config.app_name = String::from(app_name);
                    confy::store(app_name, &lxp_config).expect("Failed to write config file");
                }
                return lxp_config;
            }
        };
    }

    pub fn get_active_profile(&self) -> Option<Profile> {
        match &self.profile_active {
            Some(pa) => Some(self.profiles[pa].clone()),
            None => {
                error!("LxpConfig: no active profile found");
                None
            } // exits app
        }
    }

    pub fn get_active_profile_name(&self) -> Option<String> {
        self.profile_active.clone()
    }

    pub fn new_profile(&mut self, profile_name: &str, profile: Profile) {
        self.profiles.insert(profile_name.into(), profile);
        self.profile_active = Some(profile_name.into());
        confy::store(&self.app_name, &self).expect("Failed to write config file");
    }

    pub fn delete_all_profiles(&mut self) {
        self.profiles = HashMap::new();
        self.profile_active = None;
        confy::store(&self.app_name, &self).expect("Failed to write config file");
    }

    pub fn delete_profile(&mut self, profile_name: &str) {
        match self.profiles.remove(profile_name) {
            Some(_p) => {
                match self.profiles.keys().cloned().next() {
                    Some(pnew) => {
                        info!(
                            "Profile {} deleted, profile {} activated",
                            profile_name, pnew
                        );
                        self.profile_active = pnew.into();
                    }
                    None => {
                        info!(
                            "Profile {} deleted. No profile activated, because none is available",
                            profile_name
                        );
                        self.profile_active = None;
                    }
                };
                confy::store(&self.app_name, &self).expect("Failed to write config file");
            }
            None => error!("Could not delete profile {}: not found", profile_name), // exits app
        }
    }

    pub fn switch_profile(&mut self, profile_name: &str) {
        match self.profiles.get(profile_name) {
            Some(_v) => {
                info!("Active profile to {} switched", profile_name);
                self.profile_active = Some(profile_name.into())
            }
            None => error!("Could not switch to profile {}: not found", profile_name), // exits app
        }
        confy::store(&self.app_name, &self).expect("Failed to write config file");
    }

    pub fn show_profiles(&self) {
        match &self.profile_active {
            Some(pn) => info!("Active profile '{}'", pn),
            None => info!("<No profile active>"),
        }
        info!("\n{:<15} {:<30} {}", "<profile>", "<user>", "<url>");
        for (profile_name, profile) in &self.profiles {
            info!(
                "{:<15} {:<30} {}",
                profile_name, profile.user_name, profile.url
            );
        }
    }
}
