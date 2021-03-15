use log::{error, info, warn};
use serde_derive::{Deserialize, Serialize};
use std::error::Error;
use zbus::dbus_interface;

use crate::GetSupported;
use crate::ctrl_anime::device::{AniMePacketType, AniMeDeviceHandle};
use rog_types::anime_matrix::{
    AniMePane,
    AniMePaneBuffer,
    AniMeStatusValue,
    AniMeImageBuffer,
};
use rog_types::error::AuraError;


#[derive(Serialize, Deserialize)]
pub struct AniMeSupportedFunctions(pub bool);

impl GetSupported for CtrlAniMeDisplay {
    type A = AniMeSupportedFunctions;

    fn get_supported() -> Self::A {
        AniMeSupportedFunctions(AniMeDeviceHandle::get_supported().is_ok())
    }
}

pub struct CtrlAniMeDisplay {
    device : AniMeDeviceHandle,
}

//AnimatrixWrite
pub trait Dbus {
    /// Write an image 34x56 pixels. Each pixel is 0-255 greyscale.
    fn write_image(&self, image: AniMeImageBuffer);

    /// Write a direct stream of data
    fn write_pane(&self, pane: AniMePaneBuffer);

    fn set_on_off(&self, status: AniMeStatusValue);

    fn set_boot_on_off(&self, status: AniMeStatusValue);
}

impl crate::ZbusAdd for CtrlAniMeDisplay {
    fn add_to_server(self, server: &mut zbus::ObjectServer) {
        server
            .at("/org/asuslinux/Anime", self)
            .map_err(|err| {
                warn!("CtrlAniMeDisplay: add_to_server {}", err);
                err
            })
            .ok();
    }
}

#[dbus_interface(name = "org.asuslinux.Daemon")]
impl Dbus for CtrlAniMeDisplay {
    /// Writes a 34x56 image
    fn write_image(&self, input: AniMeImageBuffer) {
        match self.write_image_buffer(input) {
            Ok(()) => info!("Writing image to AniMe"),
            Err(err) => warn!("{}", err)
        }
    }

    /// Writes a data stream of length
    fn write_pane(&self, pane: AniMePaneBuffer) {
        let packet = AniMePacketType::WritePane(AniMePane::from(pane));
        match self.device.write_pane(packet) {
            Ok(()) => info!("Writing pane to AniMe"),
            Err(err) => warn!("{}", err)
        }
    }

    fn set_on_off(&self, status : AniMeStatusValue) {
        match self.device.write_pane(AniMePacketType::TurnOnOff(status)) {
            Ok(()) => info!("Turning {} the AniMe", status),
            Err(err) =>  warn!("{}", err)
        }
    }

    fn set_boot_on_off(&self, status: AniMeStatusValue) {
        match self.device.write_pane(AniMePacketType::TurnBootOnOff(status)) {
            Ok(()) => info!("Turning {} the AniMe at boot/shutdown", status),
            Err(err) => warn!("{}", err)
        }
    }
}

impl CtrlAniMeDisplay {
    #[inline]
    pub fn new() -> Result<CtrlAniMeDisplay, Box<dyn Error>> {
        match AniMeDeviceHandle::new() {
            Ok(device) => {
                Ok(Self { device })
            },
            Err(err) => {
                error!("Unable to initialise AniMe Device Handler: {}", err);
                Err(Box::new(err))
            }
        }
    }

    /// Write an Animatrix image
    ///
    /// The expected USB input here is *two* Vectors, 640 bytes in length. The two vectors
    /// are each one half of the full image write.
    ///
    /// After each write a flush is written, it is assumed that this tells the device to
    /// go ahead and display the written bytes
    ///
    /// # Note:
    /// The vectors are expected to contain the full sequence of bytes as follows
    ///
    /// - Write pane 1: 0x5e 0xc0 0x02 0x01 0x00 0x73 0x02 .. <led brightness>
    /// - Write pane 2: 0x5e 0xc0 0x02 0x74 0x02 0x73 0x02 .. <led brightness>
    ///
    /// Where led brightness is 0..255, low to high
    #[inline]
    fn write_image_buffer(&self, _image: AniMeImageBuffer) -> Result<(), AuraError> {
        /*
        let mut image = AniMePacketType::from(buffer);
        image[0][..7].copy_from_slice(&ANIME_PANE1_PREFIX);
        image[1][..7].copy_from_slice(&ANIME_PANE2_PREFIX);

        for row in image.iter() {
            self.write_bytes(row);
        }
        self.do_flush()?;
         */
        Ok(())
    }
}
