use argh::FromArgs;
use rog_anime::usb::{AnimAwake, AnimBooting, AnimShutdown, AnimSleeping, Brightness};
use rog_anime::AnimeType;

/// Control the AniMe matrix display
#[derive(FromArgs)]
#[argh(subcommand, name = "anime")]
pub struct AnimeCommand {
    /// override the display type
    #[argh(option)]
    pub override_type: Option<AnimeType>,
    /// enable/disable the display
    #[argh(option)]
    pub enable_display: Option<bool>,
    /// enable/disable the builtin run/powersave animation
    #[argh(option)]
    pub enable_powersave_anim: Option<bool>,
    /// set global base brightness value <Off, Low, Med, High>
    #[argh(option)]
    pub brightness: Option<Brightness>,
    /// clear the display
    #[argh(option)]
    pub clear: bool,
    /// turn the anime off when external power is unplugged
    #[argh(option)]
    pub off_when_unplugged: Option<bool>,
    /// turn the anime off when the laptop suspends
    #[argh(option)]
    pub off_when_suspended: Option<bool>,
    /// turn the anime off when the lid is closed
    #[argh(option)]
    pub off_when_lid_closed: Option<bool>,
    /// off with his head!!!
    #[argh(option)]
    pub off_with_his_head: Option<bool>,
    #[argh(subcommand)]
    pub command: Option<AnimeActions>,
}

#[derive(FromArgs)]
#[argh(subcommand)]
pub enum AnimeActions {
    /// display a PNG image
    Image(AnimeImage),
    /// display a diagonal/pixel-perfect PNG
    PixelImage(AnimeImageDiagonal),
    /// display an animated GIF
    Gif(AnimeGif),
    /// display an animated diagonal/pixel-perfect GIF
    PixelGif(AnimeGifDiagonal),
    /// change which builtin animations are shown
    SetBuiltins(Builtins),
}

/// Set the builtin animations used
#[derive(FromArgs)]
#[argh(subcommand, name = "builtins")]
pub struct Builtins {
    /// default is used if unspecified, <default:GlitchConstruction, StaticEmergence>
    #[argh(option)]
    pub boot: AnimBooting,
    /// default is used if unspecified, <default:BinaryBannerScroll, RogLogoGlitch>
    #[argh(option)]
    pub awake: AnimAwake,
    /// default is used if unspecified, <default:BannerSwipe, Starfield>
    #[argh(option)]
    pub sleep: AnimSleeping,
    /// default is used if unspecified, <default:GlitchOut, SeeYa>
    #[argh(option)]
    pub shutdown: AnimShutdown,
    /// set/apply the animations <true/false>
    #[argh(option)]
    pub set: Option<bool>,
}

/// Setup an image to be displayed on the AniMe matrix display
#[derive(FromArgs)]
#[argh(subcommand, name = "image")]
pub struct AnimeImage {
    /// full path to the png to display
    #[argh(option)]
    pub path: String,
    /// scale 1.0 == normal
    #[argh(option, default = "1.0")]
    pub scale: f32,
    /// x position (float)
    #[argh(option, default = "0.0")]
    pub x_pos: f32,
    /// y position (float)
    #[argh(option, default = "0.0")]
    pub y_pos: f32,
    /// the angle in radians
    #[argh(option, default = "0.0")]
    pub angle: f32,
    /// brightness 0.0-1.0
    #[argh(option, default = "1.0")]
    pub bright: f32,
}

/// Setup a diagonal image to be displayed. These are typically custom made to fit the exact pixels of the display
#[derive(FromArgs)]
#[argh(subcommand, name = "diagonal-image")]
pub struct AnimeImageDiagonal {
    /// full path to the png to display
    #[argh(option)]
    pub path: String,
    /// brightness 0.0-1.0
    #[argh(option, default = "1.0")]
    pub bright: f32,
}

/// Show a regular gif as an animation
#[derive(FromArgs)]
#[argh(subcommand, name = "gif")]
pub struct AnimeGif {
    /// full path to the png to display
    #[argh(option)]
    pub path: String,
    /// scale 1.0 == normal
    #[argh(option, default = "1.0")]
    pub scale: f32,
    /// x position (float)
    #[argh(option, default = "0.0")]
    pub x_pos: f32,
    /// y position (float)
    #[argh(option, default = "0.0")]
    pub y_pos: f32,
    /// the angle in radians
    #[argh(option, default = "0.0")]
    pub angle: f32,
    /// brightness 0.0-1.0
    #[argh(option, default = "1.0")]
    pub bright: f32,
    /// how many loops to play - 0 is infinite
    #[argh(option, default = "1")]
    pub loops: u32,
}

/// Setup a diagonal gif to be displayed as animation. These are typically custom made to fit the exact pixels of the display
#[derive(FromArgs)]
#[argh(subcommand, name = "diagonal-gif")]
pub struct AnimeGifDiagonal {
    /// full path to the png to display
    #[argh(option)]
    pub path: String,
    /// brightness 0.0-1.0
    #[argh(option, default = "1.0")]
    pub bright: f32,
    /// how many loops to play - 0 is infinite
    #[argh(option, default = "1")]
    pub loops: u32,
}
