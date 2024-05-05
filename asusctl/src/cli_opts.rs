use crate::anime_cli::AnimeCommand;
use crate::aura_cli::{LedBrightness, LedPowerCommand1, LedPowerCommand2, SetAuraBuiltin};
use crate::fan_curve_cli::FanCurveCommand;
use crate::slash_cli::SlashCommand;
use argh::FromArgs;
use rog_platform::platform::ThrottlePolicy;

/// Do stuff
#[derive(Default, FromArgs)]
pub struct CliStart {
    /// show program version number
    #[argh(switch)]
    pub version: bool,
    /// show supported functions of this laptop
    #[argh(switch)]
    pub show_supported: bool,
    /// set keyboard brightness <off, low, med, high>
    #[argh(option)]
    pub kbd_bright: Option<LedBrightness>,
    /// toggle to next keyboard brightness
    #[argh(switch)]
    pub next_kbd_bright: bool,
    /// toggle to previous keyboard brightness
    #[argh(switch)]
    pub prev_kbd_bright: bool,
    /// set your battery charge limit <20-100>
    #[argh(option)]
    pub chg_limit: Option<u8>,
    #[argh(subcommand)]
    pub command: Option<CliCommand>,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum CliCommand {
    LedMode(LedModeCommand),
    LedPow1(LedPowerCommand1),
    LedPow2(LedPowerCommand2),
    Profile(ProfileCommand),
    FanCurve(FanCurveCommand),
    Graphics(GraphicsCommand),
    Anime(AnimeCommand),
    Slash(SlashCommand),
    Bios(SysCommand),
}

/// Set the platform profile
#[derive(Debug, Clone, FromArgs)]
#[argh(subcommand, name = "profile")]
pub struct ProfileCommand {
    /// toggle to next profile in list
    #[argh(switch)]
    pub next: bool,
    /// list available profiles
    #[argh(switch)]
    pub list: bool,
    /// get profile
    #[argh(switch)]
    pub profile_get: bool,
    /// set the active profile
    #[argh(option)]
    pub profile_set: Option<ThrottlePolicy>,
}

/// Setup the aura device
#[derive(FromArgs)]
#[argh(subcommand, name = "aura")]
pub struct LedModeCommand {
    /// switch to next aura mode
    #[argh(switch)]
    pub next_mode: bool,
    /// switch to previous aura mode
    #[argh(switch)]
    pub prev_mode: bool,
    #[argh(subcommand)]
    pub command: Option<SetAuraBuiltin>,
}

/// Unused
#[derive(FromArgs)]
#[argh(subcommand, name = "aura")]
pub struct GraphicsCommand {}

/// Set up the system platform
#[derive(FromArgs)]
#[argh(subcommand, name = "system")]
pub struct SysCommand {
    /// set bios POST sound:  <true/false>
    #[argh(option)]
    pub post_sound_set: Option<bool>,
    /// read bios POST sound
    #[argh(switch)]
    pub post_sound_get: bool,
    /// switch GPU MUX mode: 0 = Discrete, 1 = Optimus, reboot required"
    #[argh(option)]
    pub gpu_mux_mode_set: Option<u8>,
    /// get GPU mode
    #[argh(switch)]
    pub gpu_mux_mode_get: bool,
    /// set device panel overdrive <true/false>
    #[argh(option)]
    pub panel_overdrive_set: Option<bool>,
    /// get panel overdrive
    #[argh(switch)]
    pub panel_overdrive_get: bool,
}
