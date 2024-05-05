use argh::FromArgs;
use rog_platform::platform::ThrottlePolicy;
use rog_profiles::fan_curve_set::CurveData;
use rog_profiles::FanCurvePU;

/// Set up a fan curve for the selected profile
#[derive(Debug, Clone, FromArgs)]
#[argh(subcommand, name = "curve")]
pub struct FanCurveCommand {
    /// get enabled fan profiles
    #[argh(switch)]
    pub get_enabled: bool,
    /// set the active profile's fan curve to default
    #[argh(switch)]
    pub default: bool,
    /// profile to modify fan-curve for. Shows data if no options provided
    #[argh(option)]
    pub mod_profile: Option<ThrottlePolicy>,
    /// enable or disable <true/false> fan all curves for a profile. `--mod_profile` required
    #[argh(option)]
    pub enable_fan_curves: Option<bool>,
    /// enable or disable <true/false> a single fan curve for a profile. `--mod_profile` and `--fan` required
    #[argh(option)]
    pub enable_fan_curve: Option<bool>,
    /// select fan <cpu/gpu/mid> to modify. `--mod_profile` required
    #[argh(option)]
    pub fan: Option<FanCurvePU>,
    /// data format = 30c:1%,49c:2%,59c:3%,69c:4%,79c:31%,89c:49%,99c:56%,109c:58%. `--mod-profile` required. If '%' is omitted the fan range is 0-255"
    #[argh(option)]
    pub data: Option<CurveData>,
}
