#[macro_use]
use jpegdec_sys::JPEGDRAW;
use jpegdec_sys::*;
use minifb::{Key, Window, WindowOptions};
use std::time::SystemTime;
use std::{thread, time};

// Bundle tulips.jpg in our binary
const TULIPS_CONST: &[u8; 56010] = include_bytes!("tulips.jpg");
const TULIPS_CONST_PTR: *const u8 = TULIPS_CONST.as_ptr();
//let tulipsptr = TULIPS_CONST.as_ptr();

const WIDTH: usize = 640;
const HEIGHT: usize = 480;

/// Need our framebuffer to be accessible in the C callback :(
static mut FB: Vec<u32> = Vec::new();
fn rgb565_to_rgb888(pixel: u16) -> u32 {
    let r = pixel >> 11 & (0x20 - 1);
    let g = pixel >> 5 & (0x40 - 1);
    let b = pixel & (0x20 - 1);
    let r8 = (r << 3) as u32;
    let g8 = (g << 2) as u32;
    let b8 = (b << 3) as u32;
    r8 << 16 | g8 << 8 | b8
}

static mut draw_part: u32 = 1;
static mut callback_count: u32 = 0;

fn main() {
    // Can't access static mut safely, press the "I believe" button
    unsafe {
        // Can't const init a Vec yet, so replace the FB here
        FB = vec![0; WIDTH * HEIGHT]
    }
    ///TODO: write a real draw callback
    extern "C" fn callback(p_draw: *mut JPEGDRAW) {
        unsafe {
            callback_count += 1;
            if callback_count != draw_part {
                return;
            };
        }

        let data = unsafe { *p_draw };
        let startx = data.x;
        let starty = data.y;
        let drawwidth = data.iWidth;
        let drawheight = data.iHeight;
        let pixeldata = data.pPixels;
        let bpp = data.iBpp;
        println!(
            "x {} y {} width {} height {} bpp {}",
            startx, starty, drawwidth, drawheight, bpp
        );

        // let bpp = data.iBpp;
        // let mut cur_pixel: isize = 0;
        for y in 0..drawheight {
            let yoffset = y * drawwidth;
            let y_draw_offset = (y + starty) * drawwidth;

            for x in 0..drawwidth {
                let offset = (yoffset + x) as usize;
                let draw_offset = (startx + y_draw_offset + x) as usize;
                //FB[offset] = *pixeldata.offset(offset as isize);
                unsafe {
                    let pix = rgb565_to_rgb888(*pixeldata.offset(offset as isize));
                    //let pix2 = *pixeldata.offset(offset as isize) as u32;
                    FB[draw_offset] = pix;
                }
            }
        }
    }

    const DRAW_CALLBACK: JPEG_DRAW_CALLBACK = Some(callback);
    // Create our window so we've got somewhere to put our pixels
    let mut window = Window::new(
        "JPEGDEC demo - ESC to exit",
        640,
        480,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });
    window.limit_update_rate(Some(std::time::Duration::from_micros(1_000_000 / 60)));

    let start = SystemTime::now();
    let image = Box::new(unsafe { JPEG_ZeroInitJPEGIMAGE() });
    let imgptr: *mut JPEGIMAGE = Box::into_raw(image);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        unsafe {
            window.update_with_buffer(&FB, WIDTH, HEIGHT).unwrap();
        }
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
        let sleeptime = time::Duration::from_millis(100);
        thread::sleep(sleeptime);
        unsafe {
            println!("Draw part {}", draw_part);
            draw_part += 1;
            callback_count = 0;
            if draw_part > 149 {
                draw_part = 0;
            }
        }
    }
}
