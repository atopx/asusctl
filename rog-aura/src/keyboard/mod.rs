/// All handling for `RgbAddress`ing.
mod advanced;
pub use advanced::*;

/// Helpers for consructing keyboard layouts for UI use and effects
mod layouts;
pub use layouts::*;

mod power;
use nanoserde::{DeRon, SerRon};
pub use power::*;

#[derive(Debug, Clone, PartialEq, Eq, Default, DeRon, SerRon)]
pub enum AdvancedAuraType {
    /// A `None` will apply the effect to the whole keyboard via basic-static
    /// mode
    #[default]
    None,
    Zoned(Vec<LedCode>),
    PerKey,
}
