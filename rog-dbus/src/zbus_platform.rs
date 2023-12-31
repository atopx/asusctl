//! # `DBus` interface proxy for: `org.asuslinux.Daemon`
//!
//! This code was generated by `zbus-xmlgen` `1.0.0` from `DBus` introspection
//! data. Source: `Interface '/org/asuslinux/Platform' from service
//! 'org.asuslinux.Daemon' on system bus`.
//!
//! You may prefer to adapt it, instead of using it verbatim.
//!
//! More information can be found in the
//! [Writing a client proxy](https://zeenix.pages.freedesktop.org/zbus/client.html)
//! section of the zbus documentation.
//!
//! This `DBus` object implements
//! [standard `DBus` interfaces](https://dbus.freedesktop.org/doc/dbus-specification.html),
//! (`org.freedesktop.DBus.*`) for which the following zbus proxies can be used:
//!
//! * [`zbus::fdo::PropertiesProxy`]
//! * [`zbus::fdo::PeerProxy`]
//! * [`zbus::fdo::IntrospectableProxy`]
//!
//! …consequently `zbus-xmlgen` did not generate code for the above interfaces.

use rog_platform::platform::GpuMode;
use zbus::dbus_proxy;

#[dbus_proxy(
    interface = "org.asuslinux.Daemon",
    default_path = "/org/asuslinux/Platform"
)]
trait RogBios {
    /// DgpuDisable method
    fn dgpu_disable(&self) -> zbus::Result<bool>;

    /// EgpuEnable method
    fn egpu_enable(&self) -> zbus::Result<bool>;

    /// GpuMuxMode method
    fn gpu_mux_mode(&self) -> zbus::Result<GpuMode>;

    /// PanelOd method
    fn panel_od(&self) -> zbus::Result<bool>;

    /// PostBootSound method
    fn post_boot_sound(&self) -> zbus::Result<i16>;

    /// SetDgpuDisable method
    fn set_dgpu_disable(&self, disable: bool) -> zbus::Result<()>;

    /// SetEgpuEnable method
    fn set_egpu_enable(&self, enable: bool) -> zbus::Result<()>;

    /// SetGpuMuxMode method
    fn set_gpu_mux_mode(&self, mode: GpuMode) -> zbus::Result<()>;

    /// SetPanelOd method
    fn set_panel_od(&self, overdrive: bool) -> zbus::Result<()>;

    /// SetPostBootSound method
    fn set_post_boot_sound(&self, on: bool) -> zbus::Result<()>;

    /// NotifyDgpuDisable signal
    #[dbus_proxy(signal)]
    fn notify_dgpu_disable(&self, disable: bool) -> zbus::Result<()>;

    /// NotifyEgpuEnable signal
    #[dbus_proxy(signal)]
    fn notify_egpu_enable(&self, enable: bool) -> zbus::Result<()>;

    /// NotifyGpuMuxMode signal
    #[dbus_proxy(signal)]
    fn notify_gpu_mux_mode(&self, mode: GpuMode) -> zbus::Result<()>;

    /// NotifyPanelOd signal
    #[dbus_proxy(signal)]
    fn notify_panel_od(&self, overdrive: bool) -> zbus::Result<()>;

    /// NotifyPostBootSound signal
    #[inline]
    #[dbus_proxy(signal)]
    fn notify_post_boot_sound(&self, on: bool) -> zbus::Result<()>;
}
