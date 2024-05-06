use nanoserde::{DeRon, SerRon};

use super::EffectState;
use crate::keyboard::{KeyLayout, LedCode};
use crate::{effect_state_impl, Colour};

#[derive(Debug, Clone, DeRon, SerRon)]
pub struct Static {
    led: LedCode,
    /// The starting colour
    colour: Colour,
}

impl Static {
    pub fn new(address: LedCode, colour: Colour) -> Self {
        Self {
            led: address,
            colour,
        }
    }
}

impl EffectState for Static {
    effect_state_impl!();

    fn next_colour_state(&mut self, _layout: &KeyLayout) {}
}
