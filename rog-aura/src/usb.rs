pub const LED_INIT1: [u8; 2] = [0x5d, 0xb9];
pub const LED_INIT2: &str = "]ASUS Tech.Inc."; // ] == 0x5d
pub const LED_INIT3: [u8; 6] = [0x5d, 0x05, 0x20, 0x31, 0, 0x08];
pub const LED_INIT4: &str = "^ASUS Tech.Inc."; // ^ == 0x5e
pub const LED_INIT5: [u8; 6] = [0x5e, 0x05, 0x20, 0x31, 0, 0x08];

// Only these two packets must be 17 bytes
pub const LED_APPLY: [u8; 17] = [0x5d, 0xb4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
pub const LED_SET: [u8; 17] = [0x5d, 0xb5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

pub const BOOT_MASK:i32 = 0xc31309;
pub const SLEEP_MASK:i32 = 0x300904;
pub const ALL_LEDS_MASK:i32 = 0x000002;
pub const KBD_LEDS_MASK:i32 = 0x080000;
pub const SIDE_LEDS_MASK:i32 = 0x040500;
pub const LEDS_STATE_MASK:i32 = ALL_LEDS_MASK | KBD_LEDS_MASK | SIDE_LEDS_MASK;

/// Writes out the correct byte string for brightness
pub const fn aura_brightness_bytes(brightness: u8) -> [u8; 17] {
    [
        0x5A, 0xBA, 0xC5, 0xC4, brightness, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]
}

pub const LED_AWAKE_ON_SLEEP_OFF: [u8; 17] = [
    0x5d, 0xbd, 0x01, 0xcf, 0x17, 0x0b, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub const LED_AWAKE_ON_SLEEP_ON: [u8; 17] = [
    0x5d, 0xbd, 0x01, 0xff, 0x1f, 0x0f, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub const LED_AWAKE_OFF_SLEEP_OFF: [u8; 17] = [
    0x5d, 0xbd, 0x01, 0xc3, 0x13, 0x09, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub const LED_AWAKE_OFF_SLEEP_ON: [u8; 17] = [
    0x5d, 0xbd, 0x01, 0xf3, 0x1b, 0x0d, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub const SIDE_LEDS_OFF: [u8; 17] = [
    0x5d, 0xbd, 0x01, 0x08, 0x00, 0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub const SIDE_LEDS_ON: [u8; 17] = [
    0x5d, 0xbd, 0x01, 0x0c, 0x05, 0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
