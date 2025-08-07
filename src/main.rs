use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::thread;

/// UDP转发器结构体
struct UdpForwarder {
    listen_addr: SocketAddr,
    target_addr: SocketAddr,
    buffer_size: usize,
}

impl UdpForwarder {
    /// 创建新的UDP转发器
    fn new(listen_addr: &str, target_addr: &str, buffer_size: usize) -> Result<Self, String> {
        let listen_addr = listen_addr
            .parse()
            .map_err(|e| format!("无效的监听地址: {}", e))?;
        let target_addr = target_addr
            .parse()
            .map_err(|e| format!("无效的目标地址: {}", e))?;

        Ok(Self {
            listen_addr,
            target_addr,
            buffer_size,
        })
    }

    /// 启动转发服务
    fn start(&self) -> Result<(), String> {
        // 绑定监听地址
        let socket = UdpSocket::bind(&self.listen_addr)
            .map_err(|e| format!("无法绑定到监听地址: {}", e))?;
        let socket = Arc::new(socket);
        let target_addr = self.target_addr;
        let buffer_size = self.buffer_size;

        println!(
            "开始转发 UDP 数据: {} -> {}",
            self.listen_addr, self.target_addr
        );

        // 持续接收并转发数据
        loop {
            let mut buffer = vec![0; buffer_size];
            let (bytes_read, src_addr) = socket
                .recv_from(&mut buffer)
                .map_err(|e| format!("接收数据失败: {}", e))?;

            println!(
                "从 {} 接收了 {} 字节数据，转发到 {}",
                src_addr, bytes_read, target_addr
            );

            // 转发数据到目标地址
            let socket_clone = Arc::clone(&socket);
            thread::spawn(move || {
                if let Err(e) = socket_clone.send_to(&buffer[..bytes_read], target_addr) {
                    eprintln!("转发数据失败: {}", e);
                }
            });
        }
    }
}

fn main() {
    // 命令行参数：监听地址 目标地址 [缓冲区大小]
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("用法: {} <监听地址> <目标地址> [缓冲区大小]", args[0]);
        eprintln!("示例: {} 0.0.0.0:5000 192.168.1.100:6000 4096", args[0]);
        std::process::exit(1);
    }

    let listen_addr = &args[1];
    let target_addr = &args[2];
    let buffer_size = if args.len() >= 4 {
        args[3].parse().unwrap_or(4096)
    } else {
        4096
    };

    // 创建并启动转发器
    match UdpForwarder::new(listen_addr, target_addr, buffer_size) {
        Ok(forwarder) => {
            if let Err(e) = forwarder.start() {
                eprintln!("转发服务错误: {}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("初始化转发器失败: {}", e);
            std::process::exit(1);
        }
    }
}