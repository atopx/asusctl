use crate::{
    aura_modes::{
        MultiColour, MultiColourSpeed, SingleColour, SingleColourSpeed, SingleSpeed,
        SingleSpeedDirection, TwoColourSpeed,
    },
    error::AuraError,
};
use gumdrop::Options;
use serde_derive::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Options)]
pub struct LedBrightness {
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
    type Err = AuraError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "off" => Ok(LedBrightness { level: Some(0x00) }),
            "low" => Ok(LedBrightness { level: Some(0x01) }),
            "med" => Ok(LedBrightness { level: Some(0x02) }),
            "high" => Ok(LedBrightness { level: Some(0x03) }),
            _ => {
                print!("Invalid argument, must be one of: off, low, med, high");
                Err(AuraError::ParseBrightness)
            }
        }
    }
}
impl ToString for LedBrightness {
    fn to_string(&self) -> String {
        let s = match self.level {
            Some(0x00) => "low",
            Some(0x01) => "med",
            Some(0x02) => "high",
            _ => "unknown",
        };
        s.to_string()
    }
}

/// Byte value for setting the built-in mode.
///
/// Enum corresponds to the required integer value
#[derive(Options, Deserialize, Serialize)]
pub enum SetAuraBuiltin {
    #[options(help = "set a single static colour")]
    Static(SingleColour),
    #[options(help = "pulse between one or two colours")]
    Breathe(TwoColourSpeed),
    #[options(help = "strobe through all colours")]
    Strobe(SingleSpeed),
    #[options(help = "rainbow cycling in one of four directions")]
    Rainbow(SingleSpeedDirection),
    #[options(help = "rain pattern mimicking raindrops")]
    Star(TwoColourSpeed),
    #[options(help = "rain pattern of three preset colours")]
    Rain(SingleSpeed),
    #[options(help = "pressed keys are highlighted to fade")]
    Highlight(SingleColourSpeed),
    #[options(help = "pressed keys generate horizontal laser")]
    Laser(SingleColourSpeed),
    #[options(help = "pressed keys ripple outwards like a splash")]
    Ripple(SingleColourSpeed),
    #[options(help = "set a rapid pulse")]
    Pulse(SingleColour),
    #[options(help = "set a vertical line zooming from left")]
    Comet(SingleColour),
    #[options(help = "set a wide vertical line zooming from left")]
    Flash(SingleColour),
    #[options(help = "4-zone multi-colour")]
    MultiStatic(MultiColour),
    #[options(help = "4-zone multi-colour breathing")]
    MultiBreathe(MultiColourSpeed),
}

impl Default for SetAuraBuiltin {
    fn default() -> Self {
        SetAuraBuiltin::Static(SingleColour::default())
    }
}
