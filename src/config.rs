use log::info;
use std::env;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub proxy: ProxyConfig,
    pub tasks_config: TasksConfig,
}
impl AppConfig {
    pub fn new() -> Self {
        let proxy = ProxyConfig::new();
        let tasks_config = TasksConfig::new();

        Self {
            proxy,
            tasks_config,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProxyConfig {
    // pub backends: Vec<String>,
    pub listen_addr: String,
}

impl ProxyConfig {
    pub fn new() -> Self {
        // let backends: Vec<String> = vec![];
        let listen_addr = env::var("LISTEN_ADDR").unwrap_or("0.0.0.0:4444".to_string());

        Self {
            // backends,
            listen_addr,
        }
    }
}

const DEFAULT_TASK_INTERVAL_MINUTES: u16 = 1;

#[derive(Clone, Debug)]
pub struct TasksConfig {
    pub interval_minutes: u16,
}
impl TasksConfig {
    pub fn new() -> Self {
        match std::env::var("TASK_INTERVAL_MINUTES") {
            Ok(value) => Self {
                interval_minutes: value.parse().unwrap(),
            },
            Err(_) => {
                info!(
                    "Using default task interval: {} minutes",
                    DEFAULT_TASK_INTERVAL_MINUTES
                );
                Self {
                    interval_minutes: DEFAULT_TASK_INTERVAL_MINUTES,
                }
            }
        }
    }
}
