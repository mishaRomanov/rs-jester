use std::env;

#[derive(Clone, Debug)]
pub struct ProxyConfig {
    pub backends: Vec<String>,
    pub listen_addr: String,
}

impl ProxyConfig {
    pub fn new() -> Self {
        let backends: Vec<String> = vec![];
        let listen_addr = env::var("LISTEN_ADDR").unwrap_or("0.0.0.0:4444".to_string());

        Self {
            backends,
            listen_addr,
        }
    }
}
