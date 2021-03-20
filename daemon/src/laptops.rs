use log::{info, warn};
use rog_types::aura_modes::AuraModeNum;
use serde_derive::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Read;

pub static LEDMODE_CONFIG_PATH: &str = "/etc/asusd/asusd-ledmodes.toml";
pub static HELP_ADDRESS: &str = "https://gitlab.com/asus-linux/asus-nb-ctrl";
static LAPTOP_DEVICES: [u16; 4] = [0x1866, 0x1869, 0x1854, 0x19b6];

/// A helper of sorts specifically for functions tied to laptop models
#[derive(Debug)]
pub struct LaptopBase {
    usb_product: String,
    condev_iface: Option<String>, // required for finding the Consumer Device interface
    led_support: LaptopLedData,
}

impl LaptopBase {
    pub fn usb_product(&self) -> &str {
        &self.usb_product
    }
    pub fn condev_iface(&self) -> Option<&String> {
        self.condev_iface.as_ref()
    }
    pub fn supported_modes(&self) -> &LaptopLedData {
        &self.led_support
    }
}

pub fn match_laptop() -> Option<LaptopBase> {
    for device in rusb::devices().expect("Couldn't get device").iter() {
        let device_desc = device
            .device_descriptor()
            .expect("Couldn't get device descriptor");
        if device_desc.vendor_id() == 0x0b05 && LAPTOP_DEVICES.contains(&device_desc.product_id()) {
            let prod_str = format!("{:x?}", device_desc.product_id());

            if device_desc.product_id() == 0x1854 {
                let laptop = laptop(prod_str, None);
                return Some(laptop);
            }

            let laptop = laptop(prod_str, Some("02".to_owned()));
            return Some(laptop);
        }
    }
    warn!(
        "Unsupported laptop, please request support at {}",
        HELP_ADDRESS
    );
    warn!("Continuing with minimal support");
    None
}

fn laptop(prod: String, condev_iface: Option<String>) -> LaptopBase {
    let dmi = sysfs_class::DmiId::default();
    let board_name = dmi.board_name().expect("Could not get board_name");
    let prod_family = dmi.product_family().expect("Could not get product_family");

    let mut laptop = LaptopBase {
        usb_product: prod,
        condev_iface,
        led_support: LaptopLedData {
            board_names: vec![],
            prod_family: String::new(),
            standard: vec![],
            multizone: false,
            per_key: false,
        },
    };

    if let Some(modes) = LedSupportFile::load_from_config() {
        if let Some(led_modes) = modes.matcher(&prod_family, &board_name) {
            laptop.led_support = led_modes;
            return laptop;
        }
    }
    laptop
}

pub fn print_board_info() {
    let dmi = sysfs_class::DmiId::default();
    let board_name = dmi.board_name().expect("Could not get board_name");
    let prod_name = dmi.product_name().expect("Could not get product_name");
    let prod_family = dmi.product_family().expect("Could not get product_family");

    info!("Product name: {}", prod_name.trim());
    info!("Product family: {}", prod_family.trim());
    info!("Board name: {}", board_name.trim());
}

pub fn print_modes(supported_modes: &[u8]) {
    if !supported_modes.is_empty() {
        info!("Supported Keyboard LED modes are:");
        for mode in supported_modes {
            let mode = <&str>::from(&<AuraModeNum>::from(*mode));
            info!("- {}", mode);
        }
        info!(
            "If these modes are incorrect or missing please request support at {}",
            HELP_ADDRESS
        );
    } else {
        info!("No RGB control available");
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct LedSupportFile {
    led_data: Vec<LaptopLedData>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LaptopLedData {
    pub prod_family: String,
    pub board_names: Vec<String>,
    pub standard: Vec<AuraModeNum>,
    pub multizone: bool,
    pub per_key: bool,
}

impl LedSupportFile {
    /// Consumes the LEDModes
    fn matcher(self, prod_family: &str, board_name: &str) -> Option<LaptopLedData> {
        for config in self.led_data {
            if prod_family.contains(&config.prod_family) {
                for board in &config.board_names {
                    if board_name.contains(board) {
                        info!("Matched to {} {}", config.prod_family, board);
                        return Some(config);
                    }
                }
            }
        }
        None
    }

    fn load_from_config() -> Option<Self> {
        if let Ok(mut file) = OpenOptions::new().read(true).open(&LEDMODE_CONFIG_PATH) {
            let mut buf = String::new();
            if let Ok(l) = file.read_to_string(&mut buf) {
                if l == 0 {
                    warn!("{} is empty", LEDMODE_CONFIG_PATH);
                } else {
                    return Some(toml::from_str(&buf).unwrap_or_else(|_| {
                        panic!("Could not deserialise {}", LEDMODE_CONFIG_PATH)
                    }));
                }
            }
        }
        warn!("Does {} exist?", LEDMODE_CONFIG_PATH);
        None
    }
}
