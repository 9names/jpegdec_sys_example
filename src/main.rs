use jpegdec_sys::JPEGDRAW;
use jpegdec_sys::*;
use minifb::{Key, Window, WindowOptions};
use std::time::SystemTime;

// Bundle tulips.jpg in our binary
const TULIPS_CONST: &[u8; 56010] = include_bytes!("tulips.jpg");
const TULIPS_CONST_PTR: *const u8 = TULIPS_CONST.as_ptr();

const WIDTH: usize = 640;
const HEIGHT: usize = 480;

/// Need our framebuffer to be accessible in the C callback :(
static mut FB: Vec<u32> = Vec::new();

/// Convert RGB565 (bit-packed in u16) to RGB888 (bit-packed in u32)
/// Our source JPG is RGB565, but minifb wants 24bit RGB data stored in a 32bit field
/// Note that this is using a simple shift to bring the elements into range,
/// but it will never reach full saturation (therefore color will never be quite correct)
/// Most RGB565 displays aren't that accurate anyway, but it's worth noting
fn rgb565_to_rgb888(pixel: u16) -> u32 {
    // Unpack our 5bit and 6bit RGB elements
    let r = 0b0000_0000_0001_1111 & pixel >> 11;
    let g = 0b0000_0000_0011_1111 & pixel >> 5;
    let b = 0b0000_0000_0001_1111 * pixel;
    // Shift them up so they can get close to full scale
    let r8 = (r << 3) as u32;
    let g8 = (g << 2) as u32;
    let b8 = (b << 3) as u32;
    // And bit-pack them into a u32 again
    r8 << 16 | g8 << 8 | b8
}

/// Callback function for passing to JPEGDEC that will handle drawing
extern "C" fn callback(p_draw: *mut JPEGDRAW) {
    let data = unsafe { *p_draw };
    let startx = data.x as usize;
    let starty = data.y as usize;
    let drawwidth = data.iWidth as usize;
    let drawheight = data.iHeight as usize;
    let pixeldata = data.pPixels;
    let _bpp = data.iBpp;

    for y in 0..drawheight {
        // Since we're using byte indexes into single dimension objects for display, we need to calcuate
        // how far through we are for each x/y position. This is going to depend on the width of the 
        // buffer for both the source (JPEG) and destination (framebuffer) 
        let yoffset = y * drawwidth;
        let y_draw_offset = (y + starty) * WIDTH;
        for x in 0..drawwidth {
            let offset = x + yoffset;
            let draw_offset = x + y_draw_offset + startx;
            unsafe {
                let pix = rgb565_to_rgb888(*pixeldata.add(offset));
                FB[draw_offset] = pix;
            }
        }
    }
}

fn main() {
    unsafe {
        // Can't const init a static Vec yet, so replace the FB here
        FB = vec![0; WIDTH * HEIGHT]
    }

    // Create our window so we've got somewhere to put our pixels
    let mut window = Window::new(
        "JPEGDEC demo - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });
    window.limit_update_rate(Some(std::time::Duration::from_micros(1_000_000 / 60)));

    // Set up our JPEGDEC objects
    let image = Box::new(unsafe { JPEG_ZeroInitJPEGIMAGE() });
    let imgptr: *mut JPEGIMAGE = Box::into_raw(image);
    const DRAW_CALLBACK: JPEG_DRAW_CALLBACK = Some(callback);

    // Main event loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let start = SystemTime::now();
        // Now onto the main "unsafe" stuff - decoding the image and drawing it!
        // Need an unsafe around JPEG_openRAM, JPEG_decode, and JPEG_getLastError becuase it's all C
        unsafe {
            let opened = JPEG_openRAM(
                imgptr,
                TULIPS_CONST_PTR,
                TULIPS_CONST.len() as i32,
                DRAW_CALLBACK,
            );
            if opened != 0 {
                let rc = JPEG_decode(imgptr, 0, 0, 0);
                if rc != 0 {
                    let elapsed = SystemTime::now().duration_since(start).unwrap().as_micros();
                    println!("full size decode in {} us", elapsed);
                }
                JPEG_close(imgptr);
            } else {
                let errstr = JPEG_getLastError(imgptr);
                println!("Last error: {}", errstr);
            }
        }
        // Seperate unsafe block for our buffer update - it's only unsafe because it uses FB.
        // If we could use a local buffer, Rust would be able to do it's lifetime/thread safety checks
        // and we wouldn't need this.
        unsafe {
            window.update_with_buffer(&FB, WIDTH, HEIGHT).unwrap();
        }
    }
}
