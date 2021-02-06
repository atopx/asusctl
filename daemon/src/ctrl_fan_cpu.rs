use crate::{
    config::{Config, Profile},
    GetSupported,
};
use rog_types::profile::{FanLevel, ProfileEvent};
use log::{info, warn};
use serde_derive::{Deserialize, Serialize};
use std::convert::TryInto;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use zbus::dbus_interface;
use crate::error::RogError;

static FAN_TYPE_1_PATH: &str = "/sys/devices/platform/asus-nb-wmi/throttle_thermal_policy";
static FAN_TYPE_2_PATH: &str = "/sys/devices/platform/asus-nb-wmi/fan_boost_mode";
static AMD_BOOST_PATH: &str = "/sys/devices/system/cpu/cpufreq/boost";

pub struct CtrlFanAndCPU {
    pub path: &'static str,
    config: Arc<Mutex<Config>>,
}

#[derive(Serialize, Deserialize)]
pub struct FanCpuSupportedFunctions {
    pub stock_fan_modes: bool,
    pub min_max_freq: bool,
    pub fan_curve_set: bool,
}

impl GetSupported for CtrlFanAndCPU {
    type A = FanCpuSupportedFunctions;

    fn get_supported() -> Self::A {
        FanCpuSupportedFunctions {
            stock_fan_modes: CtrlFanAndCPU::get_fan_path().is_ok(),
            min_max_freq: intel_pstate::PState::new().is_ok(),
            fan_curve_set: rog_fan_curve::Board::from_board_name().is_some(),
        }
    }
}

pub struct DbusFanAndCpu {
    inner: Arc<Mutex<CtrlFanAndCPU>>,
}

impl DbusFanAndCpu {
    pub fn new(inner: Arc<Mutex<CtrlFanAndCPU>>) -> Self {
        Self { inner }
    }
}

#[dbus_interface(name = "org.asuslinux.Daemon")]
impl DbusFanAndCpu {
    /// Set profile details
    fn set_profile(&self, profile: String) {
        if let Ok(event) = serde_json::from_str(&profile) {
            if let Ok(mut ctrl) = self.inner.try_lock() {
                if let Ok(mut cfg) = ctrl.config.clone().try_lock() {
                    cfg.read();
                    ctrl.handle_profile_event(&event, &mut cfg)
                        .unwrap_or_else(|err| warn!("{}", err));
                    self.notify_profile(&cfg.active_profile)
                        .unwrap_or(());
                }
            }
        }
    }

    /// Fetch the active profile name
    fn next_profile(&mut self) {
        if let Ok(mut ctrl) = self.inner.try_lock() {
            if let Ok(mut cfg) = ctrl.config.clone().try_lock() {
                ctrl.do_next_profile(&mut cfg)
                    .unwrap_or_else(|err| warn!("{}", err));
                if let Some(profile) = cfg.power_profiles.get(&cfg.active_profile) {
                    if let Ok(json) = serde_json::to_string(profile) {
                        self.notify_profile(&json)
                            .unwrap_or_else(|err| warn!("{}", err));
                    }
                }
            }
        }
    }

    /// Fetch the active profile name
    fn active_profile_name(&mut self) -> String {
        if let Ok(ctrl) = self.inner.try_lock() {
            if let Ok(mut cfg) = ctrl.config.try_lock() {
                cfg.read();
                return cfg.active_profile.clone();
            }
        }
        "Failed".to_string()
    }

    /// Fetch the active profile details
    fn profile(&mut self) -> String {
        if let Ok(ctrl) = self.inner.try_lock() {
            if let Ok(mut cfg) = ctrl.config.try_lock() {
                cfg.read();
                if let Some(profile) = cfg.power_profiles.get(&cfg.active_profile) {
                    if let Ok(json) = serde_json::to_string(profile) {
                        return json;
                    }
                }
            }
        }
        "Failed".to_string()
    }

    fn profiles(&mut self) -> String {
        if let Ok(ctrl) = self.inner.try_lock() {
            if let Ok(mut cfg) = ctrl.config.try_lock() {
                cfg.read();
                if let Ok(json) = serde_json::to_string(&cfg.power_profiles) {
                    return json;
                }
            }
        }
        "Failed".to_string()
    }

    #[dbus_interface(signal)]
    fn notify_profile(&self, profile: &str) -> zbus::Result<()> {}
}

impl crate::ZbusAdd for DbusFanAndCpu {
    fn add_to_server(self, server: &mut zbus::ObjectServer) {
        server
            .at(&"/org/asuslinux/Profile".try_into().unwrap(), self)
            .map_err(|err| {
                warn!("DbusFanAndCpu: add_to_server {}", err);
                err
            })
            .ok();
    }
}

impl crate::Reloadable for CtrlFanAndCPU {
    fn reload(&mut self) -> Result<(), RogError> {
        if let Ok(mut config) = self.config.clone().try_lock() {
            
            let profile = config.active_profile.clone();
            self.set(&profile, &mut config)?;
            // info!(
            //     "Reloaded fan mode: {:?}",
            //     FanLevel::from(config.power_profile)
            // );
        }
        Ok(())
    }
}

impl CtrlFanAndCPU {
    pub fn new(config: Arc<Mutex<Config>>) -> Result<Self, RogError> {
        let path = CtrlFanAndCPU::get_fan_path()?;
        info!("Device has thermal throttle control");
        Ok(CtrlFanAndCPU { path, config })
    }

    fn get_fan_path() -> Result<&'static str, RogError> {
        if Path::new(FAN_TYPE_1_PATH).exists() {
            Ok(FAN_TYPE_1_PATH)
        } else if Path::new(FAN_TYPE_2_PATH).exists() {
            Ok(FAN_TYPE_2_PATH)
        } else {
            Err(RogError::MissingFunction(
                "Fan mode not available, you may require a v5.8.10 series kernel or newer".into(),
            ))
        }
    }

    /// Toggle to next profile in list
    pub(super) fn do_next_profile(&mut self, config: &mut Config) -> Result<(), RogError> {
        config.read();

        let mut i = config
            .toggle_profiles
            .iter()
            .position(|x| x == &config.active_profile)
            .map(|i| i + 1)
            .unwrap_or(0);
        if i >= config.toggle_profiles.len() {
            i = 0;
        }

        let new_profile = config
            .toggle_profiles
            .get(i)
            .unwrap_or(&config.active_profile)
            .clone();

        self.set(&new_profile, config)?;

        info!("Profile was changed: {}", &new_profile);
        Ok(())
    }

    fn set_fan_mode(&mut self, preset: u8, config: &mut Config) -> Result<(), RogError> {
        let mode = config.active_profile.clone();
        let mut fan_ctrl = OpenOptions::new()
            .write(true)
            .open(self.path)
            .map_err(|err| RogError::Path(self.path.into(), err))?;
        config.read();
        let mut mode_config = config
            .power_profiles
            .get_mut(&mode)
            .ok_or_else(|| RogError::MissingProfile(mode.clone()))?;
        config.curr_fan_mode = preset;
        mode_config.fan_preset = preset;
        config.write();
        fan_ctrl
            .write_all(format!("{}\n", preset).as_bytes())
            .map_err(|err| RogError::Write(self.path.into(), err))?;
        info!("Fan mode set to: {:?}", FanLevel::from(preset));
        Ok(())
    }

    fn handle_profile_event(
        &mut self,
        event: &ProfileEvent,
        config: &mut Config,
    ) -> Result<(), RogError> {
        match event {
            ProfileEvent::Toggle => self.do_next_profile(config)?,
            ProfileEvent::ChangeMode(mode) => {
                self.set_fan_mode(*mode, config)?;
                let mode = config.active_profile.clone();
                self.set_pstate_for_fan_mode(&mode, config)?;
                self.set_fan_curve_for_fan_mode(&mode, config)?;
            }
            ProfileEvent::Cli(command) => {
                let profile_key = match command.profile.as_ref() {
                    Some(k) => k.clone(),
                    None => config.active_profile.clone(),
                };

                let mut profile = if command.create {
                    config
                        .power_profiles
                        .entry(profile_key.clone())
                        .or_insert_with(Profile::default)
                } else {
                    config
                        .power_profiles
                        .get_mut(&profile_key)
                        .ok_or_else(|| RogError::MissingProfile(profile_key.clone()))?
                };

                if command.turbo.is_some() {
                    profile.turbo = command.turbo.unwrap();
                }
                if let Some(min_perc) = command.min_percentage {
                    profile.min_percentage = min_perc;
                }
                if let Some(max_perc) = command.max_percentage {
                    profile.max_percentage = max_perc;
                }
                if let Some(ref preset) = command.preset {
                    profile.fan_preset = preset.into();
                }
                if let Some(ref curve) = command.curve {
                    profile.fan_curve = Some(curve.clone());
                }

                self.set(&profile_key, config)?;
            }
        }
        Ok(())
    }

    fn set(&mut self, profile: &str, config: &mut Config) -> Result<(), RogError> {
        let mode_config = config
            .power_profiles
            .get(profile)
            .ok_or_else(|| RogError::MissingProfile(profile.into()))?;
        let mut fan_ctrl = OpenOptions::new()
            .write(true)
            .open(self.path)
            .map_err(|err| RogError::Path(self.path.into(), err))?;
        config.curr_fan_mode = mode_config.fan_preset;
            fan_ctrl
            .write_all(format!("{}\n", mode_config.fan_preset).as_bytes())
            .map_err(|err| RogError::Write(self.path.into(), err))?;
        

        self.set_pstate_for_fan_mode(profile, config)?;
        self.set_fan_curve_for_fan_mode(profile, config)?;

        config.active_profile = profile.into();

        config.write();
        Ok(())
    }

    fn set_pstate_for_fan_mode(&self, mode: &str, config: &mut Config) -> Result<(), RogError> {
        info!("Setting pstate");
        let mode_config = config
            .power_profiles
            .get(mode)
            .ok_or_else(|| RogError::MissingProfile(mode.into()))?;

        // Set CPU pstate
        if let Ok(pstate) = intel_pstate::PState::new() {
            pstate.set_min_perf_pct(mode_config.min_percentage)?;
            pstate.set_max_perf_pct(mode_config.max_percentage)?;
            pstate.set_no_turbo(!mode_config.turbo)?;
            info!(
                "Intel CPU Power: min: {}%, max: {}%, turbo: {}",
                mode_config.min_percentage, mode_config.max_percentage, mode_config.turbo
            );
        } else {
            info!("Setting pstate for AMD CPU");
            // must be AMD CPU
            let mut file = OpenOptions::new()
                .write(true)
                .open(AMD_BOOST_PATH)
                .map_err(|err| RogError::Path(self.path.into(), err))?;

            let boost = if mode_config.turbo { "1" } else { "0" }; // opposite of Intel
            file.write_all(boost.as_bytes())
                .map_err(|err| RogError::Write(AMD_BOOST_PATH.into(), err))?;
            info!("AMD CPU Turbo: {}", boost);
        }
        Ok(())
    }

    fn set_fan_curve_for_fan_mode(&self, mode: &str, config: &Config) -> Result<(), RogError> {
        let mode_config = &config
            .power_profiles
            .get(mode)
            .ok_or_else(|| RogError::MissingProfile(mode.into()))?;

        if let Some(ref curve) = mode_config.fan_curve {
            use rog_fan_curve::{Board, Fan};
            if let Some(board) = Board::from_board_name() {
                curve.apply(board, Fan::Cpu)?;
                curve.apply(board, Fan::Gpu)?;
            } else {
                warn!("Fan curve unsupported on this board.")
            }
        }

        Ok(())
    }
}
