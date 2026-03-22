unsafe extern "C" {
    pub fn vm_storage_read(idx: u64, ptr: u32) -> u32;
    pub fn vm_storage_write(idx: u64, ptr: u32) -> u32;
    pub fn vm_terminal_write(ptr: u32, len: u32);
}

fn kprint(msg: &str) {
    unsafe { vm_terminal_write(msg.as_ptr() as u32, msg.len() as u32); }
}

const BLOCK_SIZE: usize = 4096;
const MAGIC: &[u8; 8] = b"TUS_VFS\0";

const FAT_BLOCK: u64 = 1;
const ROOT_DIR_BLOCK: u64 = 2;
const DATA_START_BLOCK: u64 = 3;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DirEntry {
    pub name: [u8; 32],
    pub start_block: u32,
    pub size: u32,
    pub reserved: [u8; 24],
}

impl DirEntry {
    fn empty() -> Self {
        Self {
            name: [0; 32],
            start_block: 0,
            size: 0,
            reserved: [0; 24],
        }
    }
}

pub struct Vfs;

impl Vfs {
    pub fn init() {
        kprint("[VFS] Initializing Virtual File System...\r\n");
        let mut super_block = [0u8; BLOCK_SIZE];
        unsafe {
            if vm_storage_read(0, super_block.as_mut_ptr() as u32) != 0 {
                kprint("[VFS] Failed to read Superblock. Disk might be uninitialized.\r\n");
                Self::format();
                return;
            }
        }

        if &super_block[0..8] != MAGIC {
            kprint("[VFS] Magic mismatch. Formatting disk...\r\n");
            Self::format();
        } else {
            kprint("[VFS] Disk recognized correctly.\r\n");
        }
    }

    pub fn format() {
        kprint("[VFS] Formatting disk...\r\n");
        let mut buf = [0u8; BLOCK_SIZE];
        // 1. Superblock
        buf[0..8].copy_from_slice(MAGIC);
        unsafe { vm_storage_write(0, buf.as_ptr() as u32); }

        // 2. FAT table (all 0 implies free, 0xFFFFFFFF implies EOF)
        // Set FAT[0..3] to EOF since they are reserved
        let mut fat = [0u32; 1024];
        fat[0] = 0xFFFFFFFF; // Super
        fat[1] = 0xFFFFFFFF; // FAT
        fat[2] = 0xFFFFFFFF; // RootDir
        
        let mut fat_buf = [0u8; BLOCK_SIZE];
        for i in 0..1024 {
            let bytes = fat[i].to_le_bytes();
            fat_buf[i*4..i*4+4].copy_from_slice(&bytes);
        }
        unsafe { vm_storage_write(FAT_BLOCK, fat_buf.as_ptr() as u32); }

        // 3. Root directory
        let dir_buf = [0u8; BLOCK_SIZE];
        unsafe { vm_storage_write(ROOT_DIR_BLOCK, dir_buf.as_ptr() as u32); }
        
        kprint("[VFS] Format completed.\r\n");
        
        // Setup initial directories/files
        Self::create_file("/etc/config.txt", b"TerminUS IPC Enabled\n");
    }

    fn read_fat() -> [u32; 1024] {
        let mut buf = [0u8; BLOCK_SIZE];
        unsafe { vm_storage_read(FAT_BLOCK, buf.as_mut_ptr() as u32); }
        let mut fat = [0u32; 1024];
        for i in 0..1024 {
            fat[i] = u32::from_le_bytes([buf[i*4], buf[i*4+1], buf[i*4+2], buf[i*4+3]]);
        }
        fat
    }

    fn write_fat(fat: &[u32; 1024]) {
        let mut buf = [0u8; BLOCK_SIZE];
        for i in 0..1024 {
            let bytes = fat[i].to_le_bytes();
            buf[i*4..i*4+4].copy_from_slice(&bytes);
        }
        unsafe { vm_storage_write(FAT_BLOCK, buf.as_ptr() as u32); }
    }

    fn alloc_block(fat: &mut [u32; 1024]) -> Option<u32> {
        for i in DATA_START_BLOCK as usize..1024 {
            if fat[i] == 0 {
                fat[i] = 0xFFFFFFFF; // Mark as EOF
                return Some(i as u32);
            }
        }
        None
    }

    pub fn create_file(path: &str, data: &[u8]) {
        let mut fat = Self::read_fat();
        
        // 查找或创建分配的数据块
        let mut current_block = match Self::alloc_block(&mut fat) {
            Some(b) => b,
            None => { kprint("[VFS] Error: Disk full\r\n"); return; }
        };
        let start_block = current_block;
        
        let mut data_offset = 0;
        
        while data_offset < data.len() {
            let chunk_size = core::cmp::min(data.len() - data_offset, BLOCK_SIZE);
            let mut buf = [0u8; BLOCK_SIZE];
            buf[..chunk_size].copy_from_slice(&data[data_offset..data_offset + chunk_size]);
            unsafe { vm_storage_write(current_block as u64, buf.as_ptr() as u32); }
            
            data_offset += chunk_size;
            if data_offset < data.len() {
                // alloc next
                if let Some(next) = Self::alloc_block(&mut fat) {
                    fat[current_block as usize] = next;
                    current_block = next;
                } else {
                    kprint("[VFS] Error: Disk full during write\r\n");
                    break;
                }
            }
        }
        
        Self::write_fat(&fat);
        
        // 写入 Root Dir
        let mut dir_buf = [0u8; BLOCK_SIZE];
        unsafe { vm_storage_read(ROOT_DIR_BLOCK, dir_buf.as_mut_ptr() as u32); }
        
        // 寻找空闲条目
        for i in 0..64 {
            let offset = i * 64;
            if dir_buf[offset] == 0 {
                // Found empty
                let mut name_bytes = [0u8; 32];
                let path_bytes = path.as_bytes();
                let len = core::cmp::min(32, path_bytes.len());
                name_bytes[..len].copy_from_slice(&path_bytes[..len]);
                
                dir_buf[offset..offset+32].copy_from_slice(&name_bytes);
                dir_buf[offset+32..offset+36].copy_from_slice(&start_block.to_le_bytes());
                let size = data.len() as u32;
                dir_buf[offset+36..offset+40].copy_from_slice(&size.to_le_bytes());
                
                unsafe { vm_storage_write(ROOT_DIR_BLOCK, dir_buf.as_ptr() as u32); }
                kprint("[VFS] File created.\r\n");
                return;
            }
        }
        kprint("[VFS] Error: Root directory full.\r\n");
    }

    pub fn read_file(path: &str, out_buf: &mut [u8]) -> usize {
        let mut dir_buf = [0u8; BLOCK_SIZE];
        unsafe { vm_storage_read(ROOT_DIR_BLOCK, dir_buf.as_mut_ptr() as u32); }
        
        let path_bytes = path.as_bytes();
        let mut found = false;
        let mut start_block = 0;
        let mut file_size = 0;
        
        for i in 0..64 {
            let offset = i * 64;
            if dir_buf[offset] == 0 { continue; }
            
            // 比较文件名，最大32字节
            let mut name_len = 0;
            while name_len < 32 && dir_buf[offset + name_len] != 0 {
                name_len += 1;
            }
            if name_len == path_bytes.len() && &dir_buf[offset..offset+name_len] == path_bytes {
                start_block = u32::from_le_bytes([dir_buf[offset+32], dir_buf[offset+33], dir_buf[offset+34], dir_buf[offset+35]]);
                file_size = u32::from_le_bytes([dir_buf[offset+36], dir_buf[offset+37], dir_buf[offset+38], dir_buf[offset+39]]);
                found = true;
                break;
            }
        }
        
        if !found {
            return 0;
        }
        
        let fat = Self::read_fat();
        let mut current_block = start_block;
        let mut out_offset = 0;
        let mut remaining = file_size as usize;
        
        while current_block != 0xFFFFFFFF && remaining > 0 && out_offset < out_buf.len() {
            let mut buf = [0u8; BLOCK_SIZE];
            unsafe { vm_storage_read(current_block as u64, buf.as_mut_ptr() as u32); }
            
            let chunk_size = core::cmp::min(remaining, core::cmp::min(BLOCK_SIZE, out_buf.len() - out_offset));
            out_buf[out_offset..out_offset+chunk_size].copy_from_slice(&buf[..chunk_size]);
            
            out_offset += chunk_size;
            remaining -= chunk_size;
            current_block = fat[current_block as usize];
        }
        
        out_offset
    }
}
