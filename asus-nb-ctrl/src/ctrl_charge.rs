use crate::{config::Config, error::RogError, GetSupported};
//use crate::dbus::DbusEvents;
use log::{info, warn};
use serde_derive::{Deserialize, Serialize};
use std::convert::TryInto;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use zbus::dbus_interface;

static BAT_CHARGE_PATH: &str = "/sys/class/power_supply/BAT0/charge_control_end_threshold";

#[derive(Serialize, Deserialize)]
pub struct ChargeSupportedFunctions {
    pub charge_level_set: bool,
}

impl GetSupported for CtrlCharge {
    type A = ChargeSupportedFunctions;

    fn get_supported() -> Self::A {
        ChargeSupportedFunctions {
            charge_level_set: CtrlCharge::get_battery_path().is_ok(),
        }
    }
}

pub struct CtrlCharge {
    config: Arc<Mutex<Config>>,
}

#[dbus_interface(name = "org.asuslinux.Daemon")]
impl CtrlCharge {
    pub fn set_limit(&mut self, limit: u8) {
        if let Ok(mut config) = self.config.try_lock() {
            self.set(limit, &mut config)
                .map_err(|err| {
                    warn!("CtrlCharge: set_limit {}", err);
                    err
                })
                .ok();
            self.notify_charge(limit)
                .map_err(|err| {
                    warn!("CtrlCharge: set_limit {}", err);
                    err
                })
                .ok();
        }
    }

    pub fn limit(&self) -> i8 {
        if let Ok(config) = self.config.try_lock() {
            return config.bat_charge_limit as i8;
        }
        -1
    }

    #[dbus_interface(signal)]
    pub fn notify_charge(&self, limit: u8) -> zbus::Result<()> {}
}

impl crate::ZbusAdd for CtrlCharge {
    fn add_to_server(self, server: &mut zbus::ObjectServer) {
        server
            .at(&"/org/asuslinux/Charge".try_into().unwrap(), self)
            .map_err(|err| {
                warn!("CtrlCharge: add_to_server {}", err);
                err
            })
            .ok();
    }
}

impl crate::Reloadable for CtrlCharge {
    fn reload(&mut self) -> Result<(), RogError> {
        if let Ok(mut config) = self.config.try_lock() {
            config.read();
            self.set(config.bat_charge_limit, &mut config)?;
        }
        Ok(())
    }
}

impl CtrlCharge {
    pub fn new(config: Arc<Mutex<Config>>) -> Result<Self, RogError> {
        CtrlCharge::get_battery_path()?;
        Ok(CtrlCharge { config })
    }

    fn get_battery_path() -> Result<&'static str, RogError> {
        if Path::new(BAT_CHARGE_PATH).exists() {
            Ok(BAT_CHARGE_PATH)
        } else {
            Err(RogError::MissingFunction(
                "Charge control not available, you may require a v5.8.10 series kernel or newer"
                    .into(),
            ))
        }
    }

    pub(super) fn set(&self, limit: u8, config: &mut Config) -> Result<(), RogError> {
        if limit < 20 || limit > 100 {
            warn!(
                "Unable to set battery charge limit, must be between 20-100: requested {}",
                limit
            );
        }

        let mut file = OpenOptions::new()
            .write(true)
            .open(BAT_CHARGE_PATH)
            .map_err(|err| RogError::Path(BAT_CHARGE_PATH.into(), err))?;
        file.write_all(limit.to_string().as_bytes())
            .map_err(|err| RogError::Write(BAT_CHARGE_PATH.into(), err))?;
        info!("Battery charge limit: {}", limit);

        config.read();
        config.bat_charge_limit = limit;
        config.write();

        Ok(())
    }
}
