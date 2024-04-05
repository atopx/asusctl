//! `update_and_notify` is responsible for both notifications *and* updating
//! stored statuses about the system state. This is done through either direct,
//! intoify, zbus notifications or similar methods.
//!
//! This module very much functions like a stand-alone app on its own thread.

use std::fmt::Display;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use log::{error, info, trace, warn};
use notify_rust::{Hint, Notification, NotificationHandle, Urgency};
use rog_dbus::zbus_platform::PlatformProxy;
use rog_platform::platform::GpuMode;
use serde::{Deserialize, Serialize};
use supergfxctl::actions::UserActionRequired as GfxUserAction;
use supergfxctl::pci_device::{GfxMode, GfxPower};
use supergfxctl::zbus_proxy::DaemonProxy as SuperProxy;
use tokio::time::sleep;
use zbus::export::futures_util::StreamExt;

use crate::config::Config;
use crate::error::Result;
use crate::system_state::SystemState;

const NOTIF_HEADER: &str = "ROG Control";

static mut POWER_AC_CMD: Option<Command> = None;
static mut POWER_BAT_CMD: Option<Command> = None;

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct EnabledNotifications {
    pub receive_power_states: bool,
    pub receive_notify_gfx: bool,
    pub receive_notify_gfx_status: bool,
}

impl EnabledNotifications {
    pub fn tokio_mutex(config: &Config) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(config.enabled_notifications.clone()))
    }
}

fn gpu_to_gfx(value: GpuMode) -> GfxMode {
    match value {
        GpuMode::Optimus => GfxMode::Hybrid,
        GpuMode::Integrated => GfxMode::Integrated,
        GpuMode::Egpu => GfxMode::AsusEgpu,
        GpuMode::Vfio => GfxMode::Vfio,
        GpuMode::Ultimate => GfxMode::AsusMuxDgpu,
        GpuMode::Error => GfxMode::None,
        GpuMode::NotSupported => GfxMode::None,
    }
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

        let notifs_enabled1 = $notif_enabled.clone();
        let page_states1 = $page_states.clone();

        tokio::spawn(async move {
                let conn = zbus::Connection::system().await.map_err(|e| {
                        log::error!("zbus signal: {}: {e}", stringify!($signal));
                        e
                    }).unwrap();
                let proxy = $proxy::builder(&conn).build().await.map_err(|e| {
                        log::error!("zbus signal: {}: {e}", stringify!($signal));
                        e
                    }).unwrap();
                if let Ok(mut p) = proxy.$signal().await {
                    info!("Started zbus signal thread: {}", stringify!($signal));
                    while let Some(e) = p.next().await {
                        if let Ok(out) = e.args() {
                            if let Ok(config) = notifs_enabled1.lock() {
                                if config.$signal {
                                    trace!("zbus signal {}", stringify!($signal));
                                    $notifier($msg, &out.$($out_arg)+()).ok();
                                }
                            }
                            if let Ok(mut lock) = page_states1.lock() {
                                lock.$($args)+ = *out.$($out_arg)+();
                                lock.set_notified();
                            }
                        }
                        sleep(Duration::from_millis(500)).await;
                    }
                };
            });
    };
}

pub fn start_notifications(
    config: &Config,
    page_states: &Arc<Mutex<SystemState>>,
    enabled_notifications: &Arc<Mutex<EnabledNotifications>>,
) -> Result<()> {
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

    let page_states1 = page_states.clone();
    tokio::spawn(async move {
        let conn = zbus::Connection::system()
            .await
            .map_err(|e| {
                error!("zbus signal: receive_notify_gpu_mux_mode: {e}");
                e
            })
            .unwrap();
        let proxy = PlatformProxy::new(&conn)
            .await
            .map_err(|e| {
                error!("zbus signal: receive_notify_gpu_mux_mode: {e}");
                e
            })
            .unwrap();

        let mut actual_mux_mode = GpuMode::Error;
        if let Ok(mode) = proxy.gpu_mux_mode().await {
            actual_mux_mode = GpuMode::from(mode);
        }

        info!("Started zbus signal thread: receive_notify_gpu_mux_mode");
        while let Some(e) = proxy.receive_gpu_mux_mode_changed().await.next().await {
            if let Ok(out) = e.get().await {
                let mode = GpuMode::from(out);
                if mode == actual_mux_mode {
                    continue;
                }
                if let Ok(mut lock) = page_states1.lock() {
                    lock.gfx_state.mode = gpu_to_gfx(mode);
                    lock.set_notified();
                }
                do_mux_notification("Reboot required. BIOS GPU MUX mode set to", &mode).ok();
            }
        }
    });

    use supergfxctl::pci_device::Device;
    let dev = Device::find().unwrap_or_default();
    let mut found_dgpu = false; // just for logging
    for dev in dev {
        if dev.is_dgpu() {
            let notifs_enabled1 = enabled_notifications.clone();
            let page_states1 = page_states.clone();
            // Plain old thread is perfectly fine since most of this is potentially blocking
            tokio::spawn(async move {
                let mut last_status = GfxPower::Unknown;
                loop {
                    if let Ok(status) = dev.get_runtime_status() {
                        if status != GfxPower::Unknown && status != last_status {
                            if let Ok(config) = notifs_enabled1.lock() {
                                if config.receive_notify_gfx_status {
                                    // Required check because status cycles through
                                    // active/unknown/suspended
                                    do_gpu_status_notif("dGPU status changed:", &status).ok();
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

    let page_states1 = page_states.clone();
    let enabled_notifications1 = enabled_notifications.clone();
    tokio::spawn(async move {
        let conn = zbus::Connection::system()
            .await
            .map_err(|e| {
                error!("zbus signal: receive_notify_action: {e}");
                e
            })
            .unwrap();
        let proxy = SuperProxy::builder(&conn)
            .build()
            .await
            .map_err(|e| {
                error!("zbus signal: receive_notify_action: {e}");
                e
            })
            .unwrap();

        if let Ok(mode) = proxy.mode().await {
            if let Ok(mut lock) = page_states1.lock() {
                lock.gfx_state.mode = mode;
                lock.gfx_state.has_supergfx = true;
            }
        } else {
            info!("supergfxd not running or not responding");
            return;
        }

        let page_states2 = page_states1.clone();
        recv_notif!(
            SuperProxy,
            receive_notify_gfx,
            last_notification,
            enabled_notifications1,
            page_states2,
            (gfx_state.mode),
            (mode),
            "Gfx mode changed to",
            do_notification
        );

        if let Ok(mut p) = proxy.receive_notify_action().await {
            info!("Started zbus signal thread: receive_notify_action");
            while let Some(e) = p.next().await {
                if let Ok(out) = e.args() {
                    let action = out.action();
                    let mode = if let Ok(lock) = page_states1.lock() {
                        convert_gfx_mode(lock.gfx_state.mode)
                    } else {
                        GpuMode::Error
                    };
                    match action {
                        supergfxctl::actions::UserActionRequired::Reboot => {
                            do_mux_notification("Graphics mode change requires reboot", &mode)
                        }
                        _ => do_gfx_action_notif(<&str>::from(action), *action, mode),
                    }
                    .map_err(|e| {
                        error!("zbus signal: do_gfx_action_notif: {e}");
                        e
                    })
                    .ok();
                }
            }
        };
    });

    Ok(())
}

fn convert_gfx_mode(gfx: GfxMode) -> GpuMode {
    match gfx {
        GfxMode::Hybrid => GpuMode::Optimus,
        GfxMode::Integrated => GpuMode::Integrated,
        GfxMode::NvidiaNoModeset => GpuMode::Optimus,
        GfxMode::Vfio => GpuMode::Vfio,
        GfxMode::AsusEgpu => GpuMode::Egpu,
        GfxMode::AsusMuxDgpu => GpuMode::Ultimate,
        GfxMode::None => GpuMode::Error,
    }
}

fn base_notification<T>(message: &str, data: &T) -> Notification
where
    T: Display,
{
    let mut notif = Notification::new();

    notif
        .summary(NOTIF_HEADER)
        .body(&format!("{message} {data}"))
        .timeout(-1)
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

// TODO:
fn _ac_power_notification(message: &str, on: &bool) -> Result<NotificationHandle> {
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

fn do_gfx_action_notif(message: &str, action: GfxUserAction, mode: GpuMode) -> Result<()> {
    if matches!(action, GfxUserAction::Reboot) {
        do_mux_notification("Graphics mode change requires reboot", &mode).ok();
        return Ok(());
    }

    let mut notif = Notification::new();
    notif
        .summary(NOTIF_HEADER)
        .body(&format!("Changing to {mode}. {message}"))
        .timeout(2000)
        //.hint(Hint::Resident(true))
        .hint(Hint::Category("device".into()))
        .urgency(Urgency::Critical)
        .timeout(-1)
        .icon("dialog-warning")
        .hint(Hint::Transient(true));

    if matches!(action, GfxUserAction::Logout) {
        notif.action("gfx-mode-session-action", "Logout");
        let handle = notif.show()?;
        if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
            if desktop.to_lowercase() == "gnome" {
                handle.wait_for_action(|id| {
                    if id == "gfx-mode-session-action" {
                        let mut cmd = Command::new("gnome-session-quit");
                        cmd.spawn().ok();
                    } else if id == "__closed" {
                        // TODO: cancel the switching
                    }
                });
            } else if desktop.to_lowercase() == "kde" {
                handle.wait_for_action(|id| {
                    if id == "gfx-mode-session-action" {
                        let mut cmd = Command::new("qdbus");
                        cmd.args(["org.kde.ksmserver", "/KSMServer", "logout", "1", "0", "0"]);
                        cmd.spawn().ok();
                    } else if id == "__closed" {
                        // TODO: cancel the switching
                    }
                });
            } else {
                // todo: handle alternatives
            }
        }
    } else {
        notif.show()?;
    }
    Ok(())
}

/// Actual `GpuMode` unused as data is never correct until switched by reboot
fn do_mux_notification(message: &str, m: &GpuMode) -> Result<()> {
    let mut notif = base_notification(message, &m.to_string());
    notif
        .action("gfx-mode-session-action", "Reboot")
        .urgency(Urgency::Critical)
        .icon("system-reboot-symbolic")
        .hint(Hint::Transient(true));
    let handle = notif.show()?;

    std::thread::spawn(|| {
        if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
            if desktop.to_lowercase() == "gnome" {
                handle.wait_for_action(|id| {
                    if id == "gfx-mode-session-action" {
                        let mut cmd = Command::new("gnome-session-quit");
                        cmd.arg("--reboot");
                        cmd.spawn().ok();
                    } else if id == "__closed" {
                        // TODO: cancel the switching
                    }
                });
            } else if desktop.to_lowercase() == "kde" {
                handle.wait_for_action(|id| {
                    if id == "gfx-mode-session-action" {
                        let mut cmd = Command::new("qdbus");
                        cmd.args(["org.kde.ksmserver", "/KSMServer", "logout", "1", "1", "0"]);
                        cmd.spawn().ok();
                    } else if id == "__closed" {
                        // TODO: cancel the switching
                    }
                });
            }
        }
    });
    Ok(())
}
