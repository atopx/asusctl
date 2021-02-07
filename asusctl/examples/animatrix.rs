use rog_dbus::AuraDbusClient;
use rog_types::anime_matrix::{AniMeImageBuffer, AniMePacketType, HEIGHT, WIDTH};
use tinybmp::{Bmp, Pixel};

fn main() {
    let (client, _) = AuraDbusClient::new().unwrap();

    let bmp =
        Bmp::from_slice(include_bytes!("non-skewed_r.bmp")).expect("Failed to parse BMP image");
    let pixels: Vec<Pixel> = bmp.into_iter().collect();
    //assert_eq!(pixels.len(), 56 * 56);

    // Try an outline, top and right
    let mut matrix = AniMeImageBuffer::new();

    // Aligned left
    // Conversion of color into greyscale
    for (_i, px) in pixels.iter().enumerate() {
        if (px.x as usize / 2) < WIDTH && (px.y as usize) < HEIGHT && px.x % 2 == 0 {
            let c = px.color as u32;
            matrix.get_mut()[px.y as usize][px.x as usize / 2] = c as u8;
        }
    }

    // Throw an alignment border up and right
    // {
    //     let tmp = matrix.get_mut();
    //     for x in tmp[0].iter_mut() {
    //         *x = 0xff;
    //     }
    //     for row in tmp.iter_mut() {
    //         let idx = row.len() - 1;
    //         row[idx] = 0xff;
    //     }
    // }

    client.proxies().anime().write_image(matrix).unwrap();
}
