#![no_std]
#![no_main]

use core::panic::PanicInfo;

unsafe extern "C" {
    fn vm_terminal_write(ptr: u32, len: u32);
    fn vm_gpu_open_window(ptr: u32, len: u32) -> u32;
}

fn print(msg: &str) {
    unsafe { vm_terminal_write(msg.as_ptr() as u32, msg.len() as u32); }
}

#[unsafe(no_mangle)]
pub extern "C" fn main() {
    print("--- Init Shell Started ---\r\n");
    print("[Init] Requesting a new GPU window...\r\n");
    
    let title = "TerminOS Graphic Window";
    let win_id = unsafe { vm_gpu_open_window(title.as_ptr() as u32, title.len() as u32) };
    
    if win_id != 0 {
        print("[Init] Window created successfully. ID: ");
        // (Simplified: just printing success)
        print("1\r\n");
    } else {
        print("[Init] Failed to create window.\r\n");
    }
    
    print("[Init] Shell is now waiting for events...\r\n");
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
