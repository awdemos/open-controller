use crate::linux::detect::{Compositor, detect};
use image::ImageBuffer;
use std::io::Cursor;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConnectionExt, ImageFormat};
use x11rb::rust_connection::RustConnection;

pub struct ScreenshotImage {
    pub width: u32,
    pub height: u32,
    pub png_bytes: Vec<u8>,
}

pub fn capture() -> anyhow::Result<ScreenshotImage> {
    match detect() {
        Compositor::X11 => capture_x11(),
        Compositor::Wayland => anyhow::bail!(
            "native Wayland screenshot requires a running desktop portal; run under XWayland or use an X11 session"
        ),
        Compositor::Headless => {
            anyhow::bail!("no display detected; screenshot requires X11 or Wayland")
        }
    }
}

fn capture_x11() -> anyhow::Result<ScreenshotImage> {
    let (conn, screen_num) = RustConnection::connect(None)?;
    let setup = conn.setup();
    let screen = &setup.roots[screen_num];
    let root = screen.root;
    let width = screen.width_in_pixels;
    let height = screen.height_in_pixels;

    let reply = conn
        .get_image(ImageFormat::Z_PIXMAP, root, 0, 0, width, height, !0)?
        .reply()?;

    let png = encode_png_from_zpixmap(width as u32, height as u32, reply.depth, &reply.data)?;
    Ok(ScreenshotImage {
        width: width as u32,
        height: height as u32,
        png_bytes: png,
    })
}

fn encode_png_from_zpixmap(
    width: u32,
    height: u32,
    depth: u8,
    data: &[u8],
) -> anyhow::Result<Vec<u8>> {
    if depth != 24 && depth != 32 {
        anyhow::bail!("unsupported X11 image depth: {} (expected 24 or 32)", depth);
    }

    let mut img = ImageBuffer::new(width, height);
    let bytes_per_pixel = 4usize;

    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) as usize) * bytes_per_pixel;
            if idx + 3 >= data.len() {
                break;
            }
            // Assume little-endian BGRX layout, which is common on x86 Linux X11.
            let b = data[idx];
            let g = data[idx + 1];
            let r = data[idx + 2];
            img.put_pixel(x, y, image::Rgb([r, g, b]));
        }
    }

    let mut out = Vec::new();
    img.write_to(&mut Cursor::new(&mut out), image::ImageFormat::Png)?;
    Ok(out)
}
