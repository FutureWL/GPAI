use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

/// 进程内模块注册中心:每个模块启动时 bind 随机端口并注册名字,
/// 其他模块通过名字取地址用 gRPC client 连。
#[derive(Clone, Default)]
pub struct ModuleRegistry {
    inner: Arc<Mutex<HashMap<String, SocketAddr>>>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// 绑定到 127.0.0.1 随机端口,把名字与地址注册进去,返回监听器。
    pub async fn register(&self, name: &str) -> std::io::Result<TcpListener> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        self.inner.lock().unwrap().insert(name.to_string(), addr);
        tracing::info!(module = name, %addr, "module registered");
        Ok(listener)
    }

    pub fn get(&self, name: &str) -> Option<SocketAddr> {
        self.inner.lock().unwrap().get(name).copied()
    }
}
