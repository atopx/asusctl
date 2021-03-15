use rusb::{Device, DeviceHandle, GlobalContext};
use std::time::Duration;

use crate::ctrl_anime::parser::AniMeParser;
use rog_types::anime_matrix::{AniMeStatusValue, AniMePane};

// We don't expect this ID to ever change
const DEVICE_VENDOR : u16 = 0x0b05;
const DEVICE_PRODUCT : u16 = 0x193b;

pub const PACKET_LEN : usize = 640;

// Headers of packets
const HANDSHAKE1_HEADER : [u8; 15] =
    [0x5e, b'A', b'S', b'U', b'S', b' ', b'T',
     b'E', b'C', b'H', b'.', b'I', b'N', b'C', b'.'];
const HANDSHAKE2_HEADER : [u8; 2] = [0x5e, 0xc2];
const HANDSHAKE3_HEADER : [u8; 3] = [0x5e, 0xc0, 0x04];

const TURNONOFF_HEADER : [u8; 3] = [0x5e, 0xc0, 0x04];
const TURNBOOTONOFF_HEADER : [u8; 3] = [0x5e, 0xc3, 0x01];

const APPLY_HEADER : [u8; 4] = [0x5e, 0xc4, 0x01, 0x80];

const FLUSH_HEADER : [u8; 3] = [0x5e, 0xc0, 0x03];

const PANE1_HEADER : [u8; 7] = [0x5e, 0xc0, 0x02, 0x01, 0x00, 0x73, 0x02];
const PANE2_HEADER : [u8; 7] = [0x5e, 0xc0, 0x02, 0x74, 0x02, 0x73, 0x02];

pub enum AniMePacketType {
    Handshake,
    TurnOnOff(AniMeStatusValue),
    TurnBootOnOff(AniMeStatusValue),
    WritePane(AniMePane),
}

pub type AniMePacket = [u8; PACKET_LEN];

pub struct AniMeDeviceHandle {
    device : DeviceHandle<GlobalContext>,
}
impl From<AniMePacketType> for Vec<AniMePacket> {
    #[inline]
    fn from(action : AniMePacketType) -> Self {
        let mut packets : Self;
        match action {
            AniMePacketType::Handshake => {
                packets = vec![[0u8; PACKET_LEN]; 3];
                packets[0][..15].copy_from_slice(&HANDSHAKE1_HEADER);
                packets[1][..2].copy_from_slice(&HANDSHAKE2_HEADER);
                packets[2][..3].copy_from_slice(&HANDSHAKE3_HEADER);
            },
            AniMePacketType::TurnOnOff(status) => {
                packets = vec![[0u8; PACKET_LEN]; 1];
                packets[0][..3].copy_from_slice(&TURNONOFF_HEADER);
                packets[0][3] = if bool::from(status) { 0x03 } else { 0x00 };
            },
            AniMePacketType::TurnBootOnOff(status) => {
                packets = vec![[0u8; PACKET_LEN]; 2];
                packets[0][..3].copy_from_slice(&TURNBOOTONOFF_HEADER);
                packets[0][3] = if bool::from(status) { 0x00 } else { 0x80 };
                packets[1][..4].copy_from_slice(&APPLY_HEADER);
            },
            // TODO: More informations about the algorithm in PDF or Markdown
            AniMePacketType::WritePane(pane) => {
                let pane_packets = AniMeParser::packets_from_pane(pane);
                let flush_packet = [0u8; PACKET_LEN];
                packets = vec![pane_packets.0, pane_packets.1, flush_packet];
                packets[0][..7].copy_from_slice(&PANE1_HEADER);
                packets[1][..7].copy_from_slice(&PANE2_HEADER);
                packets[2][..3].copy_from_slice(&FLUSH_HEADER);
            }
        }
        packets
    }
}
impl AniMeDeviceHandle {
    #[inline]
    pub fn new() -> rusb::Result<Self> {
        let mut device = Self::get_device(DEVICE_VENDOR, DEVICE_PRODUCT)?
            .open()?;

        device.reset()?;
        device.set_auto_detach_kernel_driver(true)?;
        device.claim_interface(0)?;
        Ok(Self { device })
    }

    #[inline]
    fn get_device(vendor : u16, product : u16)
                  -> rusb::Result<Device<GlobalContext>> {
        for device in rusb::devices()?.iter() {
            let device_desc = device.device_descriptor()?;
            if device_desc.vendor_id() == vendor &&
                device_desc.product_id() == product {
                    return Ok(device)
                }
        }
        Err(rusb::Error::NoDevice)
    }

    #[inline]
    pub fn get_supported() -> rusb::Result<Device<GlobalContext>> {
        Self::get_device(DEVICE_VENDOR, DEVICE_PRODUCT)
    }

    #[inline]
    pub fn write_packet(&self, packet : &AniMePacket) -> rusb::Result<usize> {
        self.device.write_control(
            0x21,  // request_type
            0x09,  // request
            0x35e, // value
            0x00,  // index
            packet,
            Duration::from_millis(200),
        )
    }

    #[inline]
    pub fn write_pane(&self, action : AniMePacketType) -> rusb::Result<()> {
        for packet in Vec::<AniMePacket>::from(action).iter() {
            self.write_packet(&packet)?;
        }
        Ok(())
    }
}
