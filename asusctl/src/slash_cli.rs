use argh::FromArgs;
use rog_slash::SlashMode;

/// Set up the slash thing
#[derive(FromArgs)]
#[argh(subcommand, name = "slash")]
pub struct SlashCommand {
    /// enable the Slash Ledbar
    #[argh(option)]
    pub enable: bool,
    /// disable the Slash Ledbar
    #[argh(option)]
    pub disable: bool,
    /// set brightness value <0-255>
    #[argh(option)]
    pub brightness: Option<u8>,
    /// set interval value <0-255>
    #[argh(option)]
    pub interval: Option<u8>,
    /// set SlashMode (so 'list' for all options)
    #[argh(option)]
    pub slash_mode: Option<SlashMode>,
    /// list available animations
    #[argh(switch)]
    pub list: bool,
}
