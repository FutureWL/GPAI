use gpai_core_common::ModuleRegistry;

#[tokio::test]
async fn register_then_get_returns_same_addr() {
    let reg = ModuleRegistry::new();
    let _listener = reg.register("market").await.unwrap();
    let addr = reg.get("market").expect("registered");
    assert_eq!(addr.ip().to_string(), "127.0.0.1");
    assert_ne!(addr.port(), 0);
}

#[tokio::test]
async fn get_missing_returns_none() {
    let reg = ModuleRegistry::new();
    assert!(reg.get("nope").is_none());
}
