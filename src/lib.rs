#![no_std]
#![no_main]

mod vfs;
mod ipc;

use core::panic::PanicInfo;

// --- 导入宿主机 (VM) 提供的原始 ABI ---
unsafe extern "C" {
    fn vm_terminal_write(ptr: u32, len: u32);
    fn vm_proc_spawn(ptr: u32, len: u32) -> u32;
    fn vm_gpu_open_window(ptr: u32, len: u32) -> u32;
}

fn kprint(msg: &str) {
    unsafe { vm_terminal_write(msg.as_ptr() as u32, msg.len() as u32); }
}

#[unsafe(no_mangle)]
pub extern "C" fn tus_spawn(wasm_ptr: u32, wasm_len: u32) -> u32 {
    unsafe { vm_proc_spawn(wasm_ptr, wasm_len) }
}

#[unsafe(no_mangle)]
pub extern "C" fn tus_gpu_open_window(title_ptr: u32, title_len: u32) -> u32 {
    unsafe { vm_gpu_open_window(title_ptr, title_len) }
}

// --- 内核入口点 ---
#[unsafe(no_mangle)]
pub extern "C" fn main() {
    kprint("--- TerminUS Microkernel Booting ---\r\n");
    
    // 初始化并测试 VFS
    vfs::Vfs::init();
    
    let mut buf = [0u8; 128];
    let len = vfs::Vfs::read_file("/etc/config.txt", &mut buf);
    if len > 0 {
        kprint("[Kernel] Successfully read /etc/config.txt from EVD:\r\n");
        if let Ok(content) = core::str::from_utf8(&buf[..len]) {
            kprint(content);
            kprint("\r\n");
        } else {
            kprint("[Kernel] (Non-UTF8 data read)\r\n");
        }
    } else {
        kprint("[Kernel] /etc/config.txt not found!\r\n");
    }

    // 从 VFS 加载真实的 TEP 应用
    kprint("[Kernel] Loading /bin/init.wasm from VFS...\r\n");
    let mut wasm_buf = [0u8; 8192]; // 假设应用小于 8KB
    let wasm_len = vfs::Vfs::read_file("/bin/init.wasm", &mut wasm_buf);
    
    if wasm_len > 0 {
        kprint("[Kernel] Launching /bin/init.wasm...\r\n");
        let res = tus_spawn(wasm_buf.as_ptr() as u32, wasm_len as u32);
        if res == 0 {
            kprint("[Kernel] Init process started.\r\n");
        } else {
            kprint("[Kernel] Failed to spawn init process.\r\n");
        }
    } else {
        kprint("[Kernel] /bin/init.wasm not found in VFS. Skipping TEP boot.\r\n");
    }

    // IPC 自检
    ipc::Ipc::self_test();

    kprint("[Kernel] System running.\r\n");
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    kprint("\r\n[Kernel Panic] System halted.\r\n");
    loop {}
}
