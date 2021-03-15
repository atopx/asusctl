use gumdrop::Options;
use serde_derive::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;
use zvariant_derive::Type;

use crate::error::AuraError;


pub const PANE_LEN : usize = 1214;


pub type AniMePane = [u8; PANE_LEN];
impl From<AniMePaneBuffer> for AniMePane {
    fn from(pane_buffer : AniMePaneBuffer) -> Self {
        assert!(pane_buffer.0.len() == PANE_LEN);
        let mut pane : Self = [0u8; PANE_LEN];
        for i in 0..PANE_LEN {
            pane[i] = pane_buffer.0[i];
        }
        pane
    }
}

// AniMePaneBuffer should only be used by zbus,
// Avoid using it somewhere else and prefer using AniMePane instead
#[derive(Deserialize, Serialize, Type)]
pub struct AniMePaneBuffer(Vec<u8>);
impl From<AniMePane> for AniMePaneBuffer {
    fn from(pane: AniMePane) -> Self {
        AniMePaneBuffer(pane.to_vec())
    }
}
impl AniMePaneBuffer {
    pub fn get(&self) -> &Vec<u8> {
        &self.0
    }
}

#[derive(Copy, Clone, Debug)]
#[derive(Deserialize, Serialize, Type)]
pub enum AniMeStatusValue {
    On,
    Off,
}
impl FromStr for AniMeStatusValue {
    type Err = AuraError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "on" => Ok(AniMeStatusValue::On),
            "off" => Ok(AniMeStatusValue::Off),
            _ => {
                print!("Invalid argument, must be one of: on, off");
                Err(AuraError::ParseAnime)
            }
        }
    }
}
impl From<AniMeStatusValue> for bool {
    fn from(value: AniMeStatusValue) -> Self {
        match value {
            AniMeStatusValue::On => true,
            AniMeStatusValue::Off => false,
        }
    }
}
impl Display for AniMeStatusValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>)
           -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}", if bool::from(*self) { "on" } else { "off" })
    }
}

#[derive(Options)]
pub struct AniMeLeds {
    #[options(help = "print help message")]
    help: bool,
    #[options(
        no_long,
        required,
        short = "b",
        meta = "",
        help = "set all leds brightness value"
    )]
    led_brightness: u8,
}
impl AniMeLeds {
    pub fn led_brightness(&self) -> u8 {
        self.led_brightness
    }
}

#[derive(Options)]
pub enum AniMeCommandType {
    #[options(help = "change all leds brightness")]
    Leds(AniMeLeds),
}

pub enum AniMeWriteType {
    WritePane(AniMePane),
}
impl From<AniMeCommandType> for AniMeWriteType {
    fn from(command : AniMeCommandType) -> Self {
        match command {
            AniMeCommandType::Leds(leds) => {
                Self::WritePane([leds.led_brightness; PANE_LEN])
            }
        }
    }
}




// All after this line should be removed

pub const WIDTH: usize = 34; // Width is definitely 34 items
pub const HEIGHT: usize = 56;
pub type AniMePacketType = [[u8; 640]; 2];
const BLOCK_START: usize = 7;
/// *Not* inclusive, the byte before this is the final for each "pane"
const BLOCK_END: usize = 634;
/// The length of usable data
pub const FULL_PANE_LEN: usize = (BLOCK_END - BLOCK_START) * 2;

/// Helper structure for writing images.
///
///  See the examples for ways to write an image to `AniMeMatrix` format.
#[derive(Debug, Deserialize, Serialize, Type)]
pub struct AniMeImageBuffer(Vec<Vec<u8>>);

impl Default for AniMeImageBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl AniMeImageBuffer {
    pub fn new() -> Self {
        AniMeImageBuffer(vec![vec![0u8; WIDTH]; HEIGHT])
    }

    pub fn get(&self) -> &Vec<Vec<u8>> {
        &self.0
    }

    pub fn get_mut(&mut self) -> &mut Vec<Vec<u8>> {
        &mut self.0
    }

    pub fn fill_with(&mut self, fill: u8) {
        for row in self.0.iter_mut() {
            for x in row.iter_mut() {
                *x = fill;
            }
        }
    }

    pub fn debug_print(&self) {
        // this is the index from right. It is used to progressively shorten rows
        let mut prog_row_len = WIDTH - 2;

        for (count, row) in self.0.iter().enumerate() {
            // Write the top block of LEDs (first 7 rows)
            if count < 6 {
                if count % 2 != 0 {
                    print!(" ");
                } else {
                    print!("");
                }
                let tmp = if count == 0 || count == 1 || count == 3 || count == 5 {
                    row[1..].iter()
                } else {
                    row.iter()
                };
                for _ in tmp {
                    print!(" XY");
                }

                println!();
            } else {
                // Switch to next block (looks like )
                if count % 2 != 0 {
                    // Row after 6 is only 1 less, then rows after 7 follow pattern
                    if count == 7 {
                        prog_row_len -= 1;
                    } else {
                        prog_row_len -= 2;
                    }
                } else {
                    prog_row_len += 1; // if count 6, 0
                }

                let index = row.len() - prog_row_len;

                if count % 2 == 0 {
                    print!(" ");
                }
                for (i, _) in row.iter().enumerate() {
                    if i >= index {
                        print!(" XY");
                    } else {
                        print!("   ");
                    }
                }
                println!();
            }
        }
    }
}

impl From<AniMeImageBuffer> for AniMePacketType {
    /// Do conversion from the nested Vec in AniMeMatrix to the two required
    /// packets suitable for sending over USB
    #[inline]
    fn from(anime: AniMeImageBuffer) -> Self {
        let mut buffers = [[0; 640]; 2];

        let mut write_index = BLOCK_START;
        let mut write_block = &mut buffers[0];
        let mut block1_done = false;

        // this is the index from right. It is used to progressively shorten rows
        let mut prog_row_len = WIDTH - 2;

        for (count, row) in anime.0.iter().enumerate() {
            // Write the top block of LEDs (first 7 rows)
            if count < 6 {
                for (i, x) in row.iter().enumerate() {
                    // Rows 0, 1, 3, 5 are short and misaligned
                    if count == 0 || count == 1 || count == 3 || count == 5 {
                        if i > 0 {
                            write_block[write_index - 1] = *x;
                        }
                    } else {
                        write_block[write_index] = *x;
                    }
                    write_index += 1;
                }
            } else {
                // Switch to next block (looks like )
                if count % 2 != 0 {
                    // Row after 6 is only 1 less, then rows after 7 follow pattern
                    if count == 7 {
                        prog_row_len -= 1;
                    } else {
                        prog_row_len -= 2;
                    }
                } else {
                    prog_row_len += 1; // if count 6, 0
                }

                let index = row.len() - prog_row_len;
                for n in row.iter().skip(index) {
                    // Require a special case to catch the correct end-of-packet which is
                    // 6 bytes from the end
                    if write_index == BLOCK_END && !block1_done {
                        block1_done = true;
                        write_block = &mut buffers[1];
                        write_index = BLOCK_START;
                    }

                    write_block[write_index] = *n;
                    write_index += 1;
                }
            }
        }
        buffers
    }
}

/*
#[cfg(test)]
mod tests {
    use crate::anime_matrix::*;

    use super::AniMeDataBuffer;

    #[test]
    fn check_from_data_buffer() {
        let mut data = AniMeDataBuffer::new();
        data.set([42u8; FULL_PANE_LEN]);

        let out: AniMePacketType = data.into();
    }

    #[test]
    fn check_data_alignment() {
        let mut matrix = AniMeImageBuffer::new();
        {
            let tmp = matrix.get_mut();
            for row in tmp.iter_mut() {
                let idx = row.len() - 1;
                row[idx] = 0xff;
            }
        }

        let matrix: AniMePacketType = AniMePacketType::from(matrix);

        // The bytes at the right of the initial AniMeMatrix should always end up aligned in the
        // same place after conversion to data packets

        // Check against manually worked out right align
        assert_eq!(
            matrix[0].to_vec(),
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ]
            .to_vec()
        );
        assert_eq!(
            matrix[1].to_vec(),
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0,
                0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0,
                0, 0, 0, 0
            ]
            .to_vec()
        );
    }
}
*/
