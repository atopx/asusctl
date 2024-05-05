use argh::FromArgs;
use rog_aura::error::Error;
use rog_aura::{AuraEffect, AuraModeNum, AuraZone, Colour, Direction, Speed};
use std::str::FromStr;

/// Set the aura device power options
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "aura-power")]
pub struct LedPowerCommand1 {
    /// control if LEDs enabled while awake <true/false>
    #[argh(option)]
    pub awake: Option<bool>,
    /// use with awake option, if excluded defaults to false
    #[argh(switch)]
    pub keyboard: bool,
    /// use with awake option, if excluded defaults to false
    #[argh(option)]
    pub lightbar: bool,
    /// control boot animations <true/false>
    #[argh(option)]
    pub boot: Option<bool>,
    /// control suspend animations <true/false>
    #[argh(option)]
    pub sleep: Option<bool>,
}

/// Set the aura device power options
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "aura-power")]
pub struct LedPowerCommand2 {
    #[argh(subcommand)]
    pub command: Option<SetAuraZoneEnabled>,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
pub enum SetAuraZoneEnabled {
    /// Applies to both old and new models
    Keyboard(AuraPowerStates),
    Logo(AuraPowerStates),
    Lightbar(AuraPowerStates),
    Lid(AuraPowerStates),
    RearGlow(AuraPowerStates),
}

/// Set the power states for this zone
#[derive(Debug, Clone, FromArgs)]
#[argh(subcommand, name = "states")]
pub struct AuraPowerStates {
    /// defaults to false if option unused
    #[argh(switch)]
    pub boot: bool,
    /// defaults to false if option unused
    #[argh(switch)]
    pub awake: bool,
    /// defaults to false if option unused
    #[argh(switch)]
    pub sleep: bool,
    /// defaults to false if option unused
    #[argh(switch)]
    pub shutdown: bool,
}

/// Keybaord LED brightness
#[derive(FromArgs)]
pub struct LedBrightness {
    /// led brightness level
    #[argh(option)]
    level: Option<u8>,
}
impl LedBrightness {
    pub fn new(level: Option<u8>) -> Self {
        LedBrightness { level }
    }

    pub fn level(&self) -> Option<u8> {
        self.level
    }
}
impl FromStr for LedBrightness {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "off" => Ok(LedBrightness { level: Some(0x00) }),
            "low" => Ok(LedBrightness { level: Some(0x01) }),
            "med" => Ok(LedBrightness { level: Some(0x02) }),
            "high" => Ok(LedBrightness { level: Some(0x03) }),
            _ => {
                print!("Invalid argument, must be one of: off, low, med, high");
                Err(Error::ParseBrightness)
            }
        }
    }
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for LedBrightness {
    fn to_string(&self) -> String {
        let s = match self.level {
            Some(0x00) => "low",
            Some(0x01) => "med",
            Some(0x02) => "high",
            _ => "unknown",
        };
        s.to_owned()
    }
}

/// Effect speed
#[derive(Debug, Clone, FromArgs, Default)]
#[argh(subcommand, name = "speed")]
pub struct SingleSpeed {
    /// set the speed: low, med, high
    #[argh(option)]
    pub speed: Speed,
    /// set the zone for this effect e.g, 0, 1, one, logo, lightbar-left
    #[argh(option)]
    pub zone: AuraZone,
}

/// Effect speed and direction
#[derive(Debug, Clone, FromArgs, Default)]
#[argh(subcommand, name = "speed")]
pub struct SingleSpeedDirection {
    /// set the direction: up, down, left, right
    #[argh(option)]
    pub direction: Direction,
    /// set the speed: low, med, high
    #[argh(option)]
    pub speed: Speed,
    /// set the zone for this effect e.g, 0, 1, one, logo, lightbar-left
    #[argh(option)]
    pub zone: AuraZone,
}

/// Efect colour
#[derive(Debug, Clone, Default, FromArgs)]
#[argh(subcommand, name = "colour")]
pub struct SingleColour {
    /// set the RGB value e.g, ff00ff
    #[argh(option)]
    pub colour: Colour,
    /// set the zone for this effect e.g, 0, 1, one, logo, lightbar-left
    #[argh(option)]
    pub zone: AuraZone,
}

/// Effect speed and colour
#[derive(Debug, Clone, Default, FromArgs)]
#[argh(subcommand, name = "colour")]
pub struct SingleColourSpeed {
    /// set the RGB value e.g, ff00ff
    #[argh(option)]
    pub colour: Colour,
    /// set the speed: low, med, high
    #[argh(option)]
    pub speed: Speed,
    /// set the zone for this effect e.g, 0, 1, one, logo, lightbar-left
    #[argh(option)]
    pub zone: AuraZone,
}

/// Effect colours
#[derive(Debug, Clone, FromArgs, Default)]
#[argh(subcommand, name = "colour")]
pub struct TwoColourSpeed {
    /// set the first RGB value e.g, ff00ff
    #[argh(option)]
    pub colour: Colour,
    /// set the second RGB value e.g, ff00ff
    #[argh(option)]
    pub colour2: Colour,
    /// set the speed: low, med, high
    #[argh(option)]
    pub speed: Speed,
    /// set the zone for this effect e.g, 0, 1, one, logo, lightbar-left
    #[argh(option)]
    pub zone: AuraZone,
}

/// Effect multizone colours
#[derive(Debug, Clone, Default, FromArgs)]
#[argh(subcommand, name = "zone")]
pub struct MultiZone {
    /// set the RGB value e.g, ff00ff
    #[argh(option)]
    pub colour1: Colour,
    /// set the RGB value e.g, ff00ff
    #[argh(option)]
    pub colour2: Colour,
    /// set the RGB value e.g, ff00ff
    #[argh(option)]
    pub colour3: Colour,
    /// set the RGB value e.g, ff00ff
    #[argh(option)]
    pub colour4: Colour,
}

/// Effect zone colours and speed
#[derive(Debug, Clone, Default, FromArgs)]
pub struct MultiColourSpeed {
    /// set the RGB value e.g, ff00ff
    #[argh(option)]
    pub colour1: Colour,
    /// set the RGB value e.g, ff00ff
    #[argh(option)]
    pub colour2: Colour,
    /// set the RGB value e.g, ff00ff
    #[argh(option)]
    pub colour3: Colour,
    /// set the RGB value e.g, ff00ff
    #[argh(option)]
    pub colour4: Colour,
    /// set the speed: low, med, high
    #[argh(option)]
    pub speed: Speed,
}

/// Byte value for setting the built-in mode.
///
/// Enum corresponds to the required integer value
// NOTE: The option names here must match those in rog-aura crate
#[derive(FromArgs)]
#[argh(subcommand)]
pub enum SetAuraBuiltin {
    Static(SingleColour),          // 0
    Breathe(TwoColourSpeed),       // 1
    Strobe(SingleSpeed),           // 2
    Rainbow(SingleSpeedDirection), // 3
    Stars(TwoColourSpeed),         // 4
    Rain(SingleSpeed),             // 5
    Highlight(SingleColourSpeed),  // 6
    Laser(SingleColourSpeed),      // 7
    Ripple(SingleColourSpeed),     // 8
    Pulse(SingleColour),           // 10
    Comet(SingleColour),           // 11
    Flash(SingleColour),           // 12
}

impl Default for SetAuraBuiltin {
    fn default() -> Self {
        SetAuraBuiltin::Static(SingleColour::default())
    }
}

impl From<&SingleColour> for AuraEffect {
    fn from(aura: &SingleColour) -> Self {
        Self {
            colour1: aura.colour,
            zone: aura.zone,
            ..Default::default()
        }
    }
}

impl From<&SingleSpeed> for AuraEffect {
    fn from(aura: &SingleSpeed) -> Self {
        Self {
            speed: aura.speed,
            zone: aura.zone,
            ..Default::default()
        }
    }
}

impl From<&SingleColourSpeed> for AuraEffect {
    fn from(aura: &SingleColourSpeed) -> Self {
        Self {
            colour1: aura.colour,
            speed: aura.speed,
            zone: aura.zone,
            ..Default::default()
        }
    }
}

impl From<&TwoColourSpeed> for AuraEffect {
    fn from(aura: &TwoColourSpeed) -> Self {
        Self {
            colour1: aura.colour,
            colour2: aura.colour2,
            zone: aura.zone,
            ..Default::default()
        }
    }
}

impl From<&SingleSpeedDirection> for AuraEffect {
    fn from(aura: &SingleSpeedDirection) -> Self {
        Self {
            speed: aura.speed,
            direction: aura.direction,
            zone: aura.zone,
            ..Default::default()
        }
    }
}

impl From<&SetAuraBuiltin> for AuraEffect {
    fn from(aura: &SetAuraBuiltin) -> Self {
        match aura {
            SetAuraBuiltin::Static(x) => {
                let mut data: AuraEffect = x.into();
                data.mode = AuraModeNum::Static;
                data
            }
            SetAuraBuiltin::Breathe(x) => {
                let mut data: AuraEffect = x.into();
                data.mode = AuraModeNum::Breathe;
                data
            }
            SetAuraBuiltin::Strobe(x) => {
                let mut data: AuraEffect = x.into();
                data.mode = AuraModeNum::Strobe;
                data
            }
            SetAuraBuiltin::Rainbow(x) => {
                let mut data: AuraEffect = x.into();
                data.mode = AuraModeNum::Rainbow;
                data
            }
            SetAuraBuiltin::Stars(x) => {
                let mut data: AuraEffect = x.into();
                data.mode = AuraModeNum::Star;
                data
            }
            SetAuraBuiltin::Rain(x) => {
                let mut data: AuraEffect = x.into();
                data.mode = AuraModeNum::Rain;
                data
            }
            SetAuraBuiltin::Highlight(x) => {
                let mut data: AuraEffect = x.into();
                data.mode = AuraModeNum::Highlight;
                data
            }
            SetAuraBuiltin::Laser(x) => {
                let mut data: AuraEffect = x.into();
                data.mode = AuraModeNum::Laser;
                data
            }
            SetAuraBuiltin::Ripple(x) => {
                let mut data: AuraEffect = x.into();
                data.mode = AuraModeNum::Ripple;
                data
            }
            SetAuraBuiltin::Pulse(x) => {
                let mut data: AuraEffect = x.into();
                data.mode = AuraModeNum::Pulse;
                data
            }
            SetAuraBuiltin::Comet(x) => {
                let mut data: AuraEffect = x.into();
                data.mode = AuraModeNum::Comet;
                data
            }
            SetAuraBuiltin::Flash(x) => {
                let mut data: AuraEffect = x.into();
                data.mode = AuraModeNum::Flash;
                data
            }
        }
    }
}
