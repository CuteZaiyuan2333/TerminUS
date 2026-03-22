/// TerminUS IPC (Inter-Process Communication) Protocol
///
/// 基于宿主机代理的消息队列，提供进程间通信
///
/// Message Format:
///   [0..4]  sender_pid  (u32 LE)
///   [4..8]  msg_type    (u32 LE)
///   [8..]   payload     (variable)

unsafe extern "C" {
    fn vm_terminal_write(ptr: u32, len: u32);
    fn vm_ipc_send(target_pid: u32, ptr: u32, len: u32) -> u32;
    fn vm_ipc_recv(pid: u32, buf_ptr: u32, buf_max_len: u32) -> u32;
}

fn kprint(msg: &str) {
    unsafe { vm_terminal_write(msg.as_ptr() as u32, msg.len() as u32); }
}

// 消息类型常量
pub const MSG_TYPE_PING: u32 = 1;
pub const MSG_TYPE_PONG: u32 = 2;
pub const MSG_TYPE_DATA: u32 = 3;
pub const MSG_TYPE_EXIT: u32 = 0xFF;

/// IPC 消息头
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MessageHeader {
    pub sender_pid: u32,
    pub msg_type: u32,
}

impl MessageHeader {
    pub fn new(sender_pid: u32, msg_type: u32) -> Self {
        Self { sender_pid, msg_type }
    }

    pub fn to_bytes(&self) -> [u8; 8] {
        let mut buf = [0u8; 8];
        buf[0..4].copy_from_slice(&self.sender_pid.to_le_bytes());
        buf[4..8].copy_from_slice(&self.msg_type.to_le_bytes());
        buf
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        Self {
            sender_pid: u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
            msg_type: u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
        }
    }
}

pub struct Ipc;

impl Ipc {
    /// 发送带有类型和负载的消息
    pub fn send_message(target_pid: u32, sender_pid: u32, msg_type: u32, payload: &[u8]) -> bool {
        let header = MessageHeader::new(sender_pid, msg_type);
        let header_bytes = header.to_bytes();

        // 构造完整消息: header + payload
        let total_len = 8 + payload.len();
        // 静态最大消息大小 256 字节
        if total_len > 256 {
            kprint("[IPC] Error: Message too large.\r\n");
            return false;
        }

        let mut msg_buf = [0u8; 256];
        msg_buf[0..8].copy_from_slice(&header_bytes);
        msg_buf[8..8 + payload.len()].copy_from_slice(payload);

        let result = unsafe { vm_ipc_send(target_pid, msg_buf.as_ptr() as u32, total_len as u32) };
        result == 0
    }

    /// 尝试接收一条消息，返回 (header, payload_len)
    pub fn receive_message(pid: u32, buf: &mut [u8]) -> Option<(MessageHeader, usize)> {
        let bytes_read = unsafe { vm_ipc_recv(pid, buf.as_mut_ptr() as u32, buf.len() as u32) };

        if bytes_read < 8 {
            return None;
        }

        let header = MessageHeader::from_bytes(&buf[0..8]);
        let payload_len = bytes_read as usize - 8;

        // 将 payload 移动到 buf 的前面（方便调用方直接使用 buf[0..payload_len]）
        // 注意：这里需要小心重叠拷贝
        if payload_len > 0 {
            for i in 0..payload_len {
                buf[i] = buf[8 + i];
            }
        }

        Some((header, payload_len))
    }

    /// IPC 自我测试 (Ping-Pong 回环)
    pub fn self_test() {
        kprint("[IPC] Running self-test (loopback)...\r\n");

        let payload = b"Ping";
        let success = Self::send_message(0, 0, MSG_TYPE_PING, payload);

        if !success {
            kprint("[IPC] Self-test FAILED: send_message returned error.\r\n");
            return;
        }

        let mut recv_buf = [0u8; 256];
        match Self::receive_message(0, &mut recv_buf) {
            Some((header, len)) => {
                if header.msg_type == MSG_TYPE_PING && &recv_buf[..len] == b"Ping" {
                    kprint("[IPC] Self-test PASSED: Received Ping loopback.\r\n");
                } else {
                    kprint("[IPC] Self-test FAILED: Unexpected message content.\r\n");
                }
            }
            None => {
                kprint("[IPC] Self-test FAILED: No message received.\r\n");
            }
        }
    }
}
