#![no_std]
#![no_main]

mod vfs;
mod ipc;

use core::panic::PanicInfo;

// --- 导入宿主机 (VM) 提供的原始 ABI ---
unsafe extern "C" {
    fn vm_terminal_write(ptr: u32, len: u32);
    fn vm_proc_spawn(ptr: u32, len: u32) -> u32;
}

fn kprint(msg: &str) {
    unsafe { vm_terminal_write(msg.as_ptr() as u32, msg.len() as u32); }
}

#[unsafe(no_mangle)]
pub extern "C" fn tus_spawn(wasm_ptr: u32, wasm_len: u32) -> u32 {
    unsafe { vm_proc_spawn(wasm_ptr, wasm_len) }
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

    // 模拟一个简单的 TEP 加载请求
    // 实际上这里会从 VFS 读取 .wasm 文件
    let mock_wasm = [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]; // WASM 幻数
    kprint("[Kernel] Loading mock TEP (Init shell)...\r\n");
    
    let res = tus_spawn(mock_wasm.as_ptr() as u32, mock_wasm.len() as u32);
    
    if res == 0 {
        kprint("[Kernel] Child process successfully proxied to Host.\r\n");
    } else {
        kprint("[Kernel] Failed to spawn process.\r\n");
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
