use jpegdec_sys::*;
use std::time::{SystemTime};

// Bundle tulips.jpg in our binary
const TULIPS_CONST: &[u8; 56010] = include_bytes!("tulips.jpg");
const TULIPS_CONST_PTR: *const u8 = TULIPS_CONST.as_ptr();
//let tulipsptr = TULIPS_CONST.as_ptr();

///TODO: write a real draw callback
extern "C" fn callback(_pdraw: *mut JPEGDRAW) {
    
}
// Must wrap our callback function
const DRAW_CALLBACK: JPEG_DRAW_CALLBACK = Some(callback);

fn main() {
    // Full size
    {         
        let start = SystemTime::now();
        let image = Box::new(unsafe { JPEG_ZeroInitJPEGIMAGE() });
        let imgptr: *mut JPEGIMAGE = Box::into_raw(image);
    
        // include_bytes gives us an immutable slice, copy that into a mutable one if using unmodified JPEG_openRAM
        // let tulips: &mut [u8] = &mut TULIPS_CONST.clone();
        // let tulipsptr = tulips.as_mut_ptr();
        unsafe {
            let opened = JPEG_openRAM(imgptr, TULIPS_CONST_PTR, TULIPS_CONST.len() as i32, DRAW_CALLBACK);
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
    }
    // Half size 
    {
        let start = SystemTime::now();
        let image = Box::new(unsafe { JPEG_ZeroInitJPEGIMAGE() });
        let imgptr: *mut JPEGIMAGE = Box::into_raw(image);
        unsafe {
            let opened = JPEG_openRAM(imgptr, TULIPS_CONST_PTR, TULIPS_CONST.len() as i32, DRAW_CALLBACK);
            if opened != 0 {
                let rc = JPEG_decode(imgptr, 0, 0, JPEG_SCALE_HALF as i32);
                if rc != 0 {
                    let elapsed = SystemTime::now().duration_since(start).unwrap().as_micros();
                    println!("half size decode in {} us", elapsed);
                }
                JPEG_close(imgptr);
            } else {
                let errstr = JPEG_getLastError(imgptr);
                println!("Last error: {}", errstr);
            }
        }     
    }
    // Quarter size 
    {
        let start = SystemTime::now();
        let image = Box::new(unsafe { JPEG_ZeroInitJPEGIMAGE() });
        let imgptr: *mut JPEGIMAGE = Box::into_raw(image);
        unsafe {
            let opened = JPEG_openRAM(imgptr, TULIPS_CONST_PTR, TULIPS_CONST.len() as i32, DRAW_CALLBACK);
            if opened != 0 {
                let rc = JPEG_decode(imgptr, 0, 0, JPEG_SCALE_QUARTER as i32);
                if rc != 0 {
                    let elapsed = SystemTime::now().duration_since(start).unwrap().as_micros();
                    println!("quarter size decode in {} us", elapsed);
                }
                JPEG_close(imgptr);
            } else {
                let errstr = JPEG_getLastError(imgptr);
                println!("Last error: {}", errstr);
            }
        }     
    }
}
