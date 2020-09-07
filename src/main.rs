use jpegdec_sys::JPEGDRAW;
use jpegdec_sys::*;

use embedded_graphics::{
    image::{Image, ImageRaw},
    pixelcolor::{raw::LittleEndian, Rgb565},
    prelude::*,
};

use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};

use sdl2::keyboard::Keycode;
use std::time::SystemTime;

const TULIPS_CONST: &[u8; 5184] = include_bytes!("rust-pride.jpg");

// Need our target buffer width for the callback
const IMGBUF_WIDTH: usize = 64;
const IMGBUF_HEIGHT: usize = 64;
static mut IMGBUF: [u16; 4096] = [0; 4096];

/// Callback function for passing to JPEGDEC that will handle drawing
extern "C" fn callback(p_draw: *mut JPEGDRAW) {
    let data = unsafe { *p_draw };
    let start_x = data.x as usize;
    let start_y = data.y as usize;
    let draw_width = data.iWidth as usize;
    let draw_height = data.iHeight as usize;
    let pixel_data = data.pPixels;
    // TODO: verify BPP, conditionally use different conversion function
    let _bpp = data.iBpp;

    for y in 0..draw_height {
        // Since we're using byte indexes into single dimension objects for display, we need to calcuate
        // how far through we are for each x/y position. This is going to depend on the width of the
        // buffer for both the source (JPEG) and destination (framebuffer)
        let src_y_offset = y * draw_width;
        let dst_y_offset = (y + start_y) * IMGBUF_WIDTH;
        for x in 0..draw_width {
            let src_offset = x + src_y_offset;
            let dst_offset = x + dst_y_offset + start_x;
            unsafe {
                IMGBUF[dst_offset] = *pixel_data.add(src_offset);
            }
        }
    }
}

fn main() {
    assert_eq!(unsafe { IMGBUF.len() }, IMGBUF_WIDTH * IMGBUF_HEIGHT);
    let window_size = Size::new(64, 64);

    let mut display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(window_size);
    let output_settings = OutputSettingsBuilder::new().build();
    let mut window = Window::new("JPEGDEC test", &output_settings);

    // Set up our JPEGDEC objects
    let mut image = unsafe { JPEG_ZeroInitJPEGIMAGE() };
    let imgptr: *mut JPEGIMAGE = &mut image as *mut JPEGIMAGE;

    'running: loop {
        window.update(&display);

        let start = SystemTime::now();
        unsafe {
            let opened = JPEG_openRAM(
                imgptr,
                TULIPS_CONST.as_ptr(),
                TULIPS_CONST.len() as i32,
                Some(callback),
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
        let imgbuf_u8slice = unsafe { core::mem::transmute::<[u16; 4096], [u8; 8192]>(IMGBUF) };
        let image_raw: ImageRaw<Rgb565, LittleEndian> = ImageRaw::new(&imgbuf_u8slice, 64, 64);
        let img: Image<_, Rgb565> = Image::new(&image_raw, Point::zero());
        img.draw(&mut display).unwrap();

        for event in window.events() {
            match event {
                SimulatorEvent::KeyDown { keycode, .. } => {
                    if keycode == Keycode::Escape {
                        break 'running;
                    }
                }
                SimulatorEvent::Quit => break 'running,
                _ => {}
            }
        }
    }
}
