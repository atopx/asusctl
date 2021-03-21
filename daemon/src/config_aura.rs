use crate::laptops::LaptopLedData;
use log::{error, warn};
use rog_types::aura_modes::{AuraEffect, AuraModeNum, AuraMultiZone, AuraZone};
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

pub static AURA_CONFIG_PATH: &str = "/etc/asusd/aura.conf";

#[derive(Deserialize, Serialize)]
pub struct AuraConfig {
    pub brightness: u8,
    pub current_mode: AuraModeNum,
    pub builtins: BTreeMap<AuraModeNum, AuraEffect>,
    pub multizone: Option<AuraMultiZone>,
}

impl Default for AuraConfig {
    fn default() -> Self {
        AuraConfig {
            brightness: 1,
            current_mode: AuraModeNum::Static,
            builtins: BTreeMap::new(),
            multizone: None,
        }
    }
}

impl AuraConfig {
    /// `load` will attempt to read the config, and panic if the dir is missing
    pub fn load(supported_led_modes: &LaptopLedData) -> Self {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&AURA_CONFIG_PATH)
            .unwrap_or_else(|_| {
                panic!(
                    "The file {} or directory /etc/asusd/ is missing",
                    AURA_CONFIG_PATH
                )
            }); // okay to cause panic here
        let mut buf = String::new();
        if let Ok(read_len) = file.read_to_string(&mut buf) {
            if read_len == 0 {
                return AuraConfig::create_default(&mut file, &supported_led_modes);
            } else {
                if let Ok(data) = serde_json::from_str(&buf) {
                    return data;
                }
                warn!("Could not deserialise {}", AURA_CONFIG_PATH);
                panic!("Please remove {} then restart asusd", AURA_CONFIG_PATH);
            }
        }
        AuraConfig::create_default(&mut file, &supported_led_modes)
    }

    fn create_default(file: &mut File, support_data: &LaptopLedData) -> Self {
        // create a default config here
        let mut config = AuraConfig::default();

        for n in &support_data.standard {
            config
                .builtins
                .insert(*n, AuraEffect::default_with_mode(*n));
        }

        // Should be okay to unwrap this as is since it is a Default
        let json = serde_json::to_string(&config).unwrap();
        file.write_all(json.as_bytes())
            .unwrap_or_else(|_| panic!("Could not write {}", AURA_CONFIG_PATH));
        config
    }

    pub fn read(&mut self) {
        let mut file = OpenOptions::new()
            .read(true)
            .open(&AURA_CONFIG_PATH)
            .unwrap_or_else(|err| panic!("Error reading {}: {}", AURA_CONFIG_PATH, err));
        let mut buf = String::new();
        if let Ok(l) = file.read_to_string(&mut buf) {
            if l == 0 {
                warn!("File is empty {}", AURA_CONFIG_PATH);
            } else {
                let x: AuraConfig = serde_json::from_str(&buf)
                    .unwrap_or_else(|_| panic!("Could not deserialise {}", AURA_CONFIG_PATH));
                *self = x;
            }
        }
    }

    pub fn write(&self) {
        let mut file = File::create(AURA_CONFIG_PATH).expect("Couldn't overwrite config");
        let json = serde_json::to_string_pretty(self).expect("Parse config to JSON failed");
        file.write_all(json.as_bytes())
            .unwrap_or_else(|err| error!("Could not write config: {}", err));
    }

    /// Multipurpose, will accecpt AuraEffect with zones and put in the correct store
    pub fn set_builtin(&mut self, effect: AuraEffect) {
        match effect.zone() {
            AuraZone::None => {
                self.builtins.insert(*effect.mode(), effect);
            }
            _ => {
                if let Some(multi) = self.multizone.as_mut() {
                    multi.set(effect)
                }
            }
        }
    }

    pub fn get_multizone(&self, aura_type: AuraModeNum) -> Option<&[AuraEffect; 4]> {
        if let Some(multi) = &self.multizone {
            if aura_type == AuraModeNum::Static {
                return Some(multi.static_());
            } else if aura_type == AuraModeNum::Breathe {
                return Some(multi.breathe());
            }
        }
        None
    }
}