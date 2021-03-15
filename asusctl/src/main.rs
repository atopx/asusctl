use daemon::{
    ctrl_fan_cpu::FanCpuSupportedFunctions, ctrl_leds::LedSupportedFunctions,
    ctrl_rog_bios::RogBiosSupportedFunctions, ctrl_supported::SupportedFunctions,
};
use gumdrop::{Opt, Options};
use rog_dbus::AuraDbusClient;
use rog_types::{
    anime_matrix::{AniMeStatusValue, AniMeCommandType, AniMeWriteType},
    aura_modes::AuraModes,
    cli_options::{LedBrightness, SetAuraBuiltin},
    gfx_vendors::GfxVendors,
    profile::{FanLevel, ProfileCommand, ProfileEvent},
};
use std::env::args;
use yansi_term::Colour::Green;
use yansi_term::Colour::Red;

#[derive(Default, Options)]
struct CLIStart {
    #[options(help_flag, help = "print help message")]
    help: bool,
    #[options(help = "show program version number")]
    version: bool,
    #[options(help = "show supported functions of this laptop")]
    show_supported: bool,
    #[options(meta = "", help = "<off, low, med, high>")]
    kbd_bright: Option<LedBrightness>,
    #[options(
        meta = "",
        help = "<silent, normal, boost>, set fan mode independent of profile"
    )]
    fan_mode: Option<FanLevel>,
    #[options(meta = "", help = "<20-100>")]
    chg_limit: Option<u8>,
    #[options(command)]
    command: Option<CliCommand>,
}

#[derive(Options)]
enum CliCommand {
    #[options(help = "Set the keyboard lighting from built-in modes")]
    LedMode(LedModeCommand),
    #[options(help = "Create and configure profiles")]
    Profile(ProfileCommand),
    #[options(help = "Set the graphics mode")]
    Graphics(GraphicsCommand),
    #[options(name = "anime", help = "Manage AniMe Matrix")]
    AniMe(AniMeCommand),
    #[options(help = "Change bios settings")]
    Bios(BiosCommand),
}

#[derive(Options)]
struct LedModeCommand {
    #[options(help = "print help message")]
    help: bool,
    #[options(help = "switch to next aura mode")]
    next_mode: bool,
    #[options(help = "switch to previous aura mode")]
    prev_mode: bool,
    #[options(command)]
    command: Option<SetAuraBuiltin>,
}

#[derive(Options)]
struct GraphicsCommand {
    #[options(help = "print help message")]
    help: bool,
    #[options(
        meta = "",
        help = "Set graphics mode: <nvidia, hybrid, compute, integrated>"
    )]
    mode: Option<GfxVendors>,
    #[options(help = "Get the current mode")]
    get: bool,
    #[options(help = "Get the current power status")]
    pow: bool,
    #[options(help = "Do not ask for confirmation")]
    force: bool,
}

#[derive(Options)]
struct AniMeCommand {
    #[options(help = "print help message")]
    help: bool,
    #[options(
        meta = "",
        help = "turn on/off the panel (accept/reject write requests)"
    )]
    turn: Option<AniMeStatusValue>,
    #[options(
        meta = "",
        help = "turn on/off the panel at boot (with Asus effect)"
    )]
    boot: Option<AniMeStatusValue>,
    #[options(command)]
    command: Option<AniMeCommandType>,
}

#[derive(Options, Debug)]
struct BiosCommand {
    #[options(help = "print help message")]
    help: bool,
    #[options(meta = "", no_long, help = "set bios POST sound <true/false>")]
    post_sound_set: Option<bool>,
    #[options(no_long, help = "read bios POST sound")]
    post_sound_get: bool,
    #[options(
        meta = "",
        no_long,
        help = "activate dGPU dedicated/G-Sync <true/false>"
    )]
    dedicated_gfx_set: Option<bool>,
    #[options(no_long, help = "get GPU mode")]
    dedicated_gfx_get: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = args().skip(1).collect();

    let parsed: CLIStart;
    let missing_argument_k = gumdrop::Error::missing_argument(Opt::Short('k'));
    match CLIStart::parse_args_default(&args) {
        Ok(p) => {
            parsed = p;
        }
        Err(err) if err.to_string() == missing_argument_k.to_string() => {
            parsed = CLIStart {
                kbd_bright: Some(LedBrightness::new(None)),
                ..Default::default()
            };
        }
        Err(err) => {
            eprintln!("source {}", err);
            std::process::exit(2);
        }
    }

    let (dbus, _) = AuraDbusClient::new()?;

    let supported_tmp = dbus.proxies().supported().get_supported_functions()?;
    let supported = serde_json::from_str::<SupportedFunctions>(&supported_tmp)?;

    if parsed.help {
        print_supported_help(&supported, &parsed);
        std::process::exit(1);
    }

    if parsed.version {
        println!("  asusctl v{}", env!("CARGO_PKG_VERSION"));
        println!(" rog-dbus v{}", rog_dbus::VERSION);
        println!("rog-types v{}", rog_types::VERSION);
        println!("   daemon v{}", daemon::VERSION);
        return Ok(());
    }

    match parsed.command {
        Some(CliCommand::LedMode(mode)) => handle_led_mode(&dbus, &supported.keyboard_led, &mode)?,
        Some(CliCommand::Profile(cmd)) => handle_profile(&dbus, &supported.fan_cpu_ctrl, &cmd)?,
        Some(CliCommand::Graphics(cmd)) => do_gfx(&dbus, &supported.rog_bios_ctrl, cmd)?,
        Some(CliCommand::AniMe(cmd)) => {
            if (cmd.command.is_none() && cmd.boot.is_none() && cmd.turn.is_none()) || cmd.help {
                println!("Missing arg or command\n\n{}", cmd.self_usage());
                if let Some(lst) = cmd.self_command_list() {
                    println!("\n{}", lst);
                }
            }
            else {
                if let Some(anime_turn) = cmd.turn {
                    dbus.proxies().anime().toggle_on(anime_turn.into())?
                }
                if let Some(anime_boot) = cmd.boot {
                    dbus.proxies().anime().toggle_boot_on(anime_boot.into())?
                }
                if let Some(anime_command) = cmd.command {
                    match AniMeWriteType::from(anime_command) {
                        AniMeWriteType::WritePane(pane) => {
                            dbus.proxies().anime().write_pane(pane.into())?;
                        }
                    }
                }
            }
        }
        Some(CliCommand::Bios(cmd)) => handle_bios_option(&dbus, &supported.rog_bios_ctrl, &cmd)?,
        None => {
            if (!parsed.show_supported
                && parsed.kbd_bright.is_none()
                && parsed.fan_mode.is_none()
                && parsed.chg_limit.is_none())
                || parsed.help
            {
                println!("{}", CLIStart::usage());
                println!();
                println!("{}", CLIStart::command_list().unwrap());
            }
        }
    }

    if let Some(brightness) = parsed.kbd_bright {
        match brightness.level() {
            None => {
                let level = dbus.proxies().led().get_led_brightness()?;
                println!("Current keyboard led brightness: {}", level.to_string());
            }
            Some(level) => dbus.proxies().led().set_brightness(level)?,
        }
    }

    if parsed.show_supported {
        let dat = dbus.proxies().supported().get_supported_functions()?;
        println!("Supported laptop functions:\n{}", dat);
    }

    if let Some(fan_level) = parsed.fan_mode {
        dbus.proxies().profile().write_fan_mode(fan_level.into())?;
    }
    if let Some(chg_limit) = parsed.chg_limit {
        dbus.proxies().charge().write_limit(chg_limit)?;
    }
    Ok(())
}

fn print_supported_help(supported: &SupportedFunctions, parsed: &CLIStart) {
    // As help option don't work with `parse_args_default`
    // we will call `parse_args_default_or_exit` instead
    let usage: Vec<String> = parsed.self_usage().lines().map(|s| s.to_string()).collect();
    for line in usage.iter().filter(|line| {
        if line.contains("--fan-mode") && !supported.fan_cpu_ctrl.stock_fan_modes {
            return false;
        }
        if line.contains("--chg-limit") && !supported.charge_ctrl.charge_level_set {
            return false;
        }
        true
    }) {
        println!("{}", line);
    }

    // command strings are in order of the struct
    let commands: Vec<String> = CliCommand::usage().lines().map(|s| s.to_string()).collect();
    println!("\nCommands available");
    for line in commands.iter().filter(|line| {
        if line.contains("profile")
            && !supported.fan_cpu_ctrl.stock_fan_modes
            && !supported.fan_cpu_ctrl.fan_curve_set
        {
            return false;
        }
        if line.contains("led-mode") && supported.keyboard_led.stock_led_modes.is_none() {
            return false;
        }
        if line.contains("bios")
            && (!supported.rog_bios_ctrl.dedicated_gfx_toggle
                || !supported.rog_bios_ctrl.post_sound_toggle)
        {
            return false;
        }
        if line.contains("anime") && !supported.anime_ctrl.0 {
            return false;
        }
        true
    }) {
        println!("{}", line);
    }

    if !supported.fan_cpu_ctrl.stock_fan_modes {
        println!("Note: Fan mode control is not supported by this laptop");
    }
    if !supported.charge_ctrl.charge_level_set {
        println!("Note: Charge control is not supported by this laptop");
    }
}

fn do_gfx(
    dbus: &AuraDbusClient,
    supported: &RogBiosSupportedFunctions,
    command: GraphicsCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    if command.mode.is_none() && !command.get && !command.pow && !command.force || command.help {
        println!("{}", command.self_usage());
    }

    if let Some(mode) = command.mode {
        if supported.dedicated_gfx_toggle && dbus.proxies().rog_bios().get_dedicated_gfx()? == 1 {
            println!("You can not change modes until you turn dedicated/G-Sync off and reboot");
            std::process::exit(-1);
        }

        println!(
            "Your display-manager will restart in requested mode when all users are logged out"
        );
        println!("If anything fails check `journalctl -b -u asusd`");

        dbus.proxies().gfx().gfx_write_mode(mode.into())?;
        let res = dbus.gfx_wait_changed()?;
        println!("{}", res);
        std::process::exit(0)
    }
    if command.get {
        let res = dbus.proxies().gfx().gfx_get_mode()?;
        println!("Current graphics mode: {}", res);
    }
    if command.pow {
        let res = dbus.proxies().gfx().gfx_get_pwr()?;
        if res.contains("active") {
            println!("Current power status: {}", Red.paint(&res));
        } else {
            println!("Current power status: {}", Green.paint(&res));
        }
    }
    Ok(())
}

fn handle_led_mode(
    dbus: &AuraDbusClient,
    supported: &LedSupportedFunctions,
    mode: &LedModeCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    if mode.command.is_none() && !mode.prev_mode && !mode.next_mode {
        if !mode.help {
            println!("Missing arg or command\n");
        }
        println!("{}\n", mode.self_usage());
        println!("Commands available");

        let commands: Vec<String> = LedModeCommand::command_list()
            .unwrap()
            .lines()
            .map(|s| s.to_string())
            .collect();
        for (_, command) in commands.iter().enumerate().filter(|(mode_num, _)| {
            if let Some(modes) = supported.stock_led_modes.as_ref() {
                return modes.contains(&(*mode_num as u8));
            }
            false
        }) {
            println!("{}", command);
        }

        println!("\nHelp can also be requested on modes, e.g: static --help");
        return Ok(());
    }

    if mode.next_mode && mode.prev_mode {
        println!("Please specify either next or previous");
        return Ok(());
    }
    if mode.next_mode {
        dbus.proxies().led().next_led_mode()?;
    } else if mode.prev_mode {
        dbus.proxies().led().prev_led_mode()?;
    } else if let Some(mode) = mode.command.as_ref() {
        if mode.help_requested() {
            println!("{}", mode.self_usage());
            return Ok(());
        }
        dbus.proxies()
            .led()
            .set_led_mode(&<AuraModes>::from(mode))?;
    }
    Ok(())
}

fn handle_profile(
    dbus: &AuraDbusClient,
    supported: &FanCpuSupportedFunctions,
    cmd: &ProfileCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    if !cmd.next
        && !cmd.create
        && cmd.curve.is_none()
        && cmd.max_percentage.is_none()
        && cmd.min_percentage.is_none()
        && cmd.fan_preset.is_none()
        && cmd.profile.is_none()
        && cmd.turbo.is_none()
    {
        if !cmd.help {
            println!("Missing arg or command\n");
        }
        let usage: Vec<String> = ProfileCommand::usage()
            .lines()
            .map(|s| s.to_string())
            .collect();
        for line in usage
            .iter()
            .filter(|line| !line.contains("--curve") || supported.fan_curve_set)
        {
            println!("{}", line);
        }

        if let Some(lst) = cmd.self_command_list() {
            println!("\n{}", lst);
        }
        std::process::exit(1);
    }
    if cmd.next {
        dbus.proxies().profile().next_fan()?;
    } else {
        dbus.proxies()
            .profile()
            .write_command(&ProfileEvent::Cli(cmd.clone()))?
    }

    Ok(())
}

fn handle_bios_option(
    dbus: &AuraDbusClient,
    supported: &RogBiosSupportedFunctions,
    cmd: &BiosCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    {
        if (cmd.dedicated_gfx_set.is_none()
            && !cmd.dedicated_gfx_get
            && cmd.post_sound_set.is_none()
            && !cmd.post_sound_get)
            || cmd.help
        {
            println!("Missing arg or command\n");

            let usage: Vec<String> = BiosCommand::usage()
                .lines()
                .map(|s| s.to_string())
                .collect();

            for line in usage.iter().filter(|line| {
                !(line.contains("sound") && !supported.post_sound_toggle)
                    || !(line.contains("GPU") && !supported.dedicated_gfx_toggle)
            }) {
                println!("{}", line);
            }
        }

        if let Some(opt) = cmd.post_sound_set {
            dbus.proxies().rog_bios().set_post_sound(opt)?;
        }
        if cmd.post_sound_get {
            let res = dbus.proxies().rog_bios().get_post_sound()? == 1;
            println!("Bios POST sound on: {}", res);
        }
        if let Some(opt) = cmd.dedicated_gfx_set {
            println!("Rebuilding initrd to include drivers");
            dbus.proxies().rog_bios().set_dedicated_gfx(opt)?;
            println!("The mode change is not active until you reboot, on boot the bios will make the required change");
            if opt {
                println!(
                    "NOTE: on reboot your display manager will be forced to use Nvidia drivers"
                );
            } else {
                println!("NOTE: after reboot you can then select regular graphics modes");
            }
        }
        if cmd.dedicated_gfx_get {
            let res = dbus.proxies().rog_bios().get_dedicated_gfx()? == 1;
            println!("Bios dedicated GPU on: {}", res);
        }
    }
    Ok(())
}
