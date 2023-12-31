//! `update_and_notify` is responsible for both notifications *and* updating
//! stored statuses about the system state. This is done through either direct,
//! intoify, zbus notifications or similar methods.

use std::fmt::Display;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use log::{error, info, trace, warn};
use notify_rust::{Hint, Notification, NotificationHandle, Urgency};
use rog_dbus::zbus_anime::AnimeProxy;
use rog_dbus::zbus_led::LedProxy;
use rog_dbus::zbus_platform::RogBiosProxy;
use rog_dbus::zbus_power::PowerProxy;
use rog_dbus::zbus_profile::ProfileProxy;
use rog_platform::platform::GpuMode;
use rog_profiles::Profile;
use serde::{Deserialize, Serialize};
use supergfxctl::pci_device::GfxPower;
use supergfxctl::zbus_proxy::DaemonProxy as SuperProxy;
use tokio::time::sleep;
use zbus::export::futures_util::{future, StreamExt};

use crate::config::Config;
use crate::error::Result;
use crate::system_state::SystemState;

const NOTIF_HEADER: &str = "ROG Control";

static mut POWER_AC_CMD: Option<Command> = None;
static mut POWER_BAT_CMD: Option<Command> = None;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct EnabledNotifications {
    pub receive_notify_post_boot_sound: bool,
    pub receive_notify_panel_od: bool,
    pub receive_notify_dgpu_disable: bool,
    pub receive_notify_egpu_enable: bool,
    pub receive_notify_gpu_mux_mode: bool,
    pub receive_notify_charge_control_end_threshold: bool,
    pub receive_notify_mains_online: bool,
    pub receive_notify_profile: bool,
    pub receive_notify_led: bool,
    /// Anime
    pub receive_power_states: bool,
    pub receive_notify_gfx: bool,
    pub receive_notify_gfx_status: bool,
    pub all_enabled: bool,
}

impl Default for EnabledNotifications {
    fn default() -> Self {
        Self {
            receive_notify_post_boot_sound: false,
            receive_notify_panel_od: true,
            receive_notify_dgpu_disable: true,
            receive_notify_egpu_enable: true,
            receive_notify_gpu_mux_mode: true,
            receive_notify_charge_control_end_threshold: true,
            receive_notify_mains_online: false,
            receive_notify_profile: true,
            receive_notify_led: true,
            receive_power_states: false,
            receive_notify_gfx: false,
            receive_notify_gfx_status: false,
            all_enabled: false,
        }
    }
}

impl EnabledNotifications {
    pub fn tokio_mutex(config: &Config) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(config.enabled_notifications.clone()))
    }
}

macro_rules! notify {
    ($notifier:expr, $last_notif:ident) => {
        if let Some(notif) = $last_notif.take() {
            notif.close();
        }
        if let Ok(x) = $notifier {
            $last_notif.replace(x);
        }
    };
}

// TODO: drop the macro and use generics plus closure
macro_rules! recv_notif {
    ($proxy:ident,
        $signal:ident,
        $last_notif:ident,
        $notif_enabled:ident,
        $page_states:ident,
        ($($args: tt)*),
        ($($out_arg:tt)+),
        $msg:literal,
        $notifier:ident) => {

        let last_notif = $last_notif.clone();
        let notifs_enabled1 = $notif_enabled.clone();
        let page_states1 = $page_states.clone();

        tokio::spawn(async move {
                let conn = zbus::Connection::system().await.map_err(|e| {
                        log::error!("zbus signal: {}: {e}", stringify!($signal));
                        e
                    }).unwrap();
                let proxy = $proxy::new(&conn).await.map_err(|e| {
                        log::error!("zbus signal: {}: {e}", stringify!($signal));
                        e
                    }).unwrap();
                if let Ok(mut p) = proxy.$signal().await {
                    info!("Started zbus signal thread: {}", stringify!($signal));
                    while let Some(e) = p.next().await {
                        if let Ok(out) = e.args() {
                            if let Ok(config) = notifs_enabled1.lock() {
                                if config.all_enabled && config.$signal {
                                    if let Ok(ref mut lock) = last_notif.lock() {
                                        trace!("zbus signal {} locked last_notif", stringify!($signal));
                                        notify!($notifier($msg, &out.$($out_arg)+()), lock);
                                    }
                                }
                            }
                            if let Ok(mut lock) = page_states1.lock() {
                                lock.$($args)+ = *out.$($out_arg)+();
                                lock.set_notified();
                            }
                        }
                    }
                };
            });
    };
}

type SharedHandle = Arc<Mutex<Option<NotificationHandle>>>;

pub fn start_notifications(
    config: &Config,
    page_states: &Arc<Mutex<SystemState>>,
    enabled_notifications: &Arc<Mutex<EnabledNotifications>>,
) -> Result<()> {
    let last_notification: SharedHandle = Arc::new(Mutex::new(None));

    // Setup the AC/BAT commands that will run on poweer status change
    unsafe {
        let prog: Vec<&str> = config.ac_command.split_whitespace().collect();
        if prog.len() > 1 {
            let mut cmd = Command::new(prog[0]);

            for arg in prog.iter().skip(1) {
                cmd.arg(*arg);
            }
            POWER_AC_CMD = Some(cmd);
        }
    }
    unsafe {
        let prog: Vec<&str> = config.bat_command.split_whitespace().collect();
        if prog.len() > 1 {
            let mut cmd = Command::new(prog[0]);

            for arg in prog.iter().skip(1) {
                cmd.arg(*arg);
            }
            POWER_BAT_CMD = Some(cmd);
        }
    }

    // BIOS notif
    recv_notif!(
        RogBiosProxy,
        receive_notify_post_boot_sound,
        last_notification,
        enabled_notifications,
        page_states,
        (bios.post_sound),
        (on),
        "BIOS Post sound",
        do_notification
    );

    recv_notif!(
        RogBiosProxy,
        receive_notify_panel_od,
        last_notification,
        enabled_notifications,
        page_states,
        (bios.panel_overdrive),
        (overdrive),
        "Panel Overdrive enabled:",
        do_notification
    );

    recv_notif!(
        RogBiosProxy,
        receive_notify_dgpu_disable,
        last_notification,
        enabled_notifications,
        page_states,
        (bios.dgpu_disable),
        (disable),
        "BIOS dGPU disabled",
        do_notification
    );

    recv_notif!(
        RogBiosProxy,
        receive_notify_egpu_enable,
        last_notification,
        enabled_notifications,
        page_states,
        (bios.egpu_enable),
        (enable),
        "BIOS eGPU enabled",
        do_notification
    );

    // Charge notif
    recv_notif!(
        PowerProxy,
        receive_notify_charge_control_end_threshold,
        last_notification,
        enabled_notifications,
        page_states,
        (power_state.charge_limit),
        (limit),
        "Battery charge limit changed to",
        do_notification
    );

    recv_notif!(
        PowerProxy,
        receive_notify_mains_online,
        last_notification,
        enabled_notifications,
        page_states,
        (power_state.ac_power),
        (on),
        "AC Power power is",
        ac_power_notification
    );

    // Profile notif
    recv_notif!(
        ProfileProxy,
        receive_notify_profile,
        last_notification,
        enabled_notifications,
        page_states,
        (profiles.current),
        (profile),
        "Profile changed to",
        do_thermal_notif
    );
    // notify!(do_thermal_notif(&out.profile), lock);

    // LED notif
    recv_notif!(
        LedProxy,
        receive_notify_led,
        last_notification,
        enabled_notifications,
        page_states,
        (aura.current_mode),
        (data.mode),
        "Keyboard LED mode changed to",
        do_notification
    );

    let page_states1 = page_states.clone();
    tokio::spawn(async move {
        let conn = zbus::Connection::system()
            .await
            .map_err(|e| {
                error!("zbus signal: receive_power_states: {e}");
                e
            })
            .unwrap();
        let proxy = AnimeProxy::new(&conn)
            .await
            .map_err(|e| {
                error!("zbus signal: receive_power_states: {e}");
                e
            })
            .unwrap();
        if let Ok(p) = proxy.receive_power_states().await {
            info!("Started zbus signal thread: receive_power_states");
            p.for_each(|_| {
                if let Ok(_lock) = page_states1.lock() {
                    // TODO: lock.anime.
                }
                future::ready(())
            })
            .await;
        };
    });

    let page_states1 = page_states.clone();
    let last_notification1 = last_notification.clone();
    tokio::spawn(async move {
        let conn = zbus::Connection::system()
            .await
            .map_err(|e| {
                error!("zbus signal: receive_notify_gpu_mux_mode: {e}");
                e
            })
            .unwrap();
        let proxy = RogBiosProxy::new(&conn)
            .await
            .map_err(|e| {
                error!("zbus signal: receive_notify_gpu_mux_mode: {e}");
                e
            })
            .unwrap();

        let mut actual_mux_mode = GpuMode::Error;
        if let Ok(mode) = proxy.gpu_mux_mode().await {
            actual_mux_mode = mode;
        }

        if let Ok(mut p) = proxy.receive_notify_gpu_mux_mode().await {
            info!("Started zbus signal thread: receive_power_states");
            while let Some(e) = p.next().await {
                if let Ok(out) = e.args() {
                    if out.mode == actual_mux_mode {
                        continue;
                    }
                    if let Ok(mut lock) = page_states1.lock() {
                        lock.bios.dedicated_gfx = out.mode;
                        lock.set_notified();
                    }
                    if let Ok(ref mut lock) = last_notification1.lock() {
                        if let Some(notif) = lock.take() {
                            notif.close();
                        }
                    }
                    do_mux_notification("Reboot required. BIOS GPU MUX mode set to", &out.mode)
                        .ok();
                }
            }
        };
    });

    if let Ok(lock) = page_states.try_lock() {
        use supergfxctl::pci_device::Device;
        let dev = Device::find().unwrap_or_default();
        let mut found_dgpu = false; // just for logging
        for dev in dev {
            if dev.is_dgpu() {
                let notifs_enabled1 = enabled_notifications.clone();
                let last_notif = last_notification.clone();
                let page_states1 = page_states.clone();
                // Plain old thread is perfectly fine since most of this is potentially blocking
                tokio::spawn(async move {
                    let mut last_status = GfxPower::Unknown;
                    loop {
                        if let Ok(status) = dev.get_runtime_status() {
                            if status != GfxPower::Unknown && status != last_status {
                                if let Ok(config) = notifs_enabled1.lock() {
                                    if config.all_enabled && config.receive_notify_gfx_status {
                                        // Required check because status cycles through
                                        // active/unknown/suspended
                                        if let Ok(ref mut lock) = last_notif.lock() {
                                            notify!(
                                                do_gpu_status_notif(
                                                    "dGPU status changed:",
                                                    &status
                                                ),
                                                lock
                                            );
                                        }
                                    }
                                }
                                if let Ok(mut lock) = page_states1.lock() {
                                    lock.set_notified();
                                }
                            }
                            if let Ok(mut lock) = page_states1.lock() {
                                lock.gfx_state.power_status = status;
                            }
                            last_status = status;
                        }
                        sleep(Duration::from_millis(500)).await;
                    }
                });
                found_dgpu = true;
                break;
            }
        }
        if !found_dgpu {
            warn!("Did not find a dGPU on this system, dGPU status won't be avilable");
        }

        if lock.gfx_state.has_supergfx {
            recv_notif!(
                SuperProxy,
                receive_notify_gfx,
                last_notification,
                enabled_notifications,
                page_states,
                (gfx_state.mode),
                (mode),
                "Gfx mode changed to",
                do_notification
            );

            tokio::spawn(async move {
                let conn = zbus::Connection::system()
                    .await
                    .map_err(|e| {
                        error!("zbus signal: receive_notify_action: {e}");
                        e
                    })
                    .unwrap();
                let proxy = SuperProxy::new(&conn)
                    .await
                    .map_err(|e| {
                        error!("zbus signal: receive_notify_action: {e}");
                        e
                    })
                    .unwrap();
                if let Ok(mut p) = proxy.receive_notify_action().await {
                    info!("Started zbus signal thread: receive_notify_action");
                    while let Some(e) = p.next().await {
                        if let Ok(out) = e.args() {
                            let action = out.action();
                            do_gfx_action_notif(
                                "Gfx mode change requires",
                                &format!("{action:?}",),
                            )
                            .map_err(|e| {
                                error!("zbus signal: do_gfx_action_notif: {e}");
                                e
                            })
                            .unwrap();
                        }
                    }
                };
            });
        }
    }

    Ok(())
}

fn base_notification<T>(message: &str, data: &T) -> Notification
where
    T: Display,
{
    let mut notif = Notification::new();

    notif
        .summary(NOTIF_HEADER)
        .body(&format!("{message} {data}"))
        .timeout(2000)
        //.hint(Hint::Resident(true))
        .hint(Hint::Category("device".into()));

    notif
}

fn do_notification<T>(message: &str, data: &T) -> Result<NotificationHandle>
where
    T: Display,
{
    Ok(base_notification(message, data).show()?)
}

fn ac_power_notification(message: &str, on: &bool) -> Result<NotificationHandle> {
    let data = if *on {
        unsafe {
            if let Some(cmd) = POWER_AC_CMD.as_mut() {
                if let Err(e) = cmd.spawn() {
                    error!("AC power command error: {e}");
                }
            }
        }
        "plugged".to_owned()
    } else {
        unsafe {
            if let Some(cmd) = POWER_BAT_CMD.as_mut() {
                if let Err(e) = cmd.spawn() {
                    error!("Battery power command error: {e}");
                }
            }
        }
        "unplugged".to_owned()
    };
    Ok(base_notification(message, &data).show()?)
}

fn do_thermal_notif(message: &str, profile: &Profile) -> Result<NotificationHandle> {
    let icon = match profile {
        Profile::Balanced => "asus_notif_yellow",
        Profile::Performance => "asus_notif_red",
        Profile::Quiet => "asus_notif_green",
    };
    let profile: &str = (*profile).into();
    let mut notif = base_notification(message, &profile.to_uppercase());
    Ok(notif.icon(icon).show()?)
}

fn do_gpu_status_notif(message: &str, data: &GfxPower) -> Result<NotificationHandle> {
    // eww
    let mut notif = base_notification(message, &<&str>::from(data).to_owned());
    let icon = match data {
        GfxPower::Suspended => "asus_notif_blue",
        GfxPower::Off => "asus_notif_green",
        GfxPower::AsusDisabled => "asus_notif_white",
        GfxPower::AsusMuxDiscreet | GfxPower::Active => "asus_notif_red",
        GfxPower::Unknown => "gpu-integrated",
    };
    notif.icon(icon);
    Ok(Notification::show(&notif)?)
}

fn do_gfx_action_notif<T>(message: &str, data: &T) -> Result<()>
where
    T: Display,
{
    let mut notif = base_notification(message, data);
    notif.action("gnome-session-quit", "Logout");
    notif.urgency(Urgency::Critical);
    notif.timeout(3000);
    notif.icon("dialog-warning");
    notif.hint(Hint::Transient(true));
    let handle = notif.show()?;
    handle.wait_for_action(|id| {
        if id == "gnome-session-quit" {
            let mut cmd = Command::new("gnome-session-quit");
            cmd.spawn().ok();
        } else if id == "__closed" {
            // TODO: cancel the switching
        }
    });
    Ok(())
}

/// Actual `GpuMode` unused as data is never correct until switched by reboot
fn do_mux_notification(message: &str, m: &GpuMode) -> Result<()> {
    let mut notif = base_notification(message, &m.to_string());
    notif.action("gnome-session-quit", "Reboot");
    notif.urgency(Urgency::Critical);
    notif.icon("system-reboot-symbolic");
    notif.hint(Hint::Transient(true));
    let handle = notif.show()?;

    std::thread::spawn(|| {
        handle.wait_for_action(|id| {
            if id == "gnome-session-quit" {
                let mut cmd = Command::new("gnome-session-quit");
                cmd.arg("--reboot");
                cmd.spawn().ok();
            } else if id == "__closed" {
                // TODO: cancel the switching
            }
        });
    });
    Ok(())
}
