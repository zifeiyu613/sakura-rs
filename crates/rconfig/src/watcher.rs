use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use std::thread;
use std::collections::HashMap;
use crate::{AppConfig, ConfigBuilder, ConfigError};

pub trait ConfigChangeObserver: Send + Sync {
    fn on_config_changed(&self, old_config: &AppConfig, new_config: &AppConfig);
}

pub struct ConfigWatcher {
    config: Arc<RwLock<AppConfig>>,
    builder: ConfigBuilder,
    file_paths: Vec<PathBuf>,
    check_interval: Duration,
    file_mtimes: Arc<Mutex<HashMap<PathBuf, std::time::SystemTime>>>,
    observers: Vec<Box<dyn ConfigChangeObserver>>,
    running: Arc<RwLock<bool>>,
}

impl ConfigWatcher {
    pub fn new(config: AppConfig, builder: ConfigBuilder) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            builder,
            file_paths: Vec::new(),
            check_interval: Duration::from_secs(30),
            file_mtimes: Arc::new(Mutex::new(HashMap::new())),
            observers: Vec::new(),
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub fn watch_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        let path_buf = path.as_ref().to_path_buf();
        self.file_paths.push(path_buf);
        self
    }

    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.check_interval = interval;
        self
    }

    pub fn add_observer<O: ConfigChangeObserver + 'static>(mut self, observer: O) -> Self {
        self.observers.push(Box::new(observer));
        self
    }

    // 启动监控线程
    pub fn start(self) -> ConfigWatcherHandle {
        // 初始化文件修改时间
        {
            let mut mtimes = self.file_mtimes.lock().unwrap();
            for path in &self.file_paths {
                if let Ok(metadata) = std::fs::metadata(path) {
                    if let Ok(mtime) = metadata.modified() {
                        mtimes.insert(path.clone(), mtime);
                    }
                }
            }
        }

        // 设置为运行状态
        {
            let mut running = self.running.write().unwrap();
            *running = true;
        }

        // 获取需要在线程中使用的资源引用
        let config = Arc::clone(&self.config);
        let file_mtimes = Arc::clone(&self.file_mtimes);
        let running = Arc::clone(&self.running);
        let file_paths = self.file_paths.clone();
        let check_interval = self.check_interval;
        let mut builder = self.builder;
        let observers = Arc::new(self.observers);

        // 启动监控线程
        let thread_handle = thread::spawn(move || {
            while *running.read().unwrap() {
                let mut config_changed = false;

                // 检查文件是否被修改
                for path in &file_paths {
                    if !path.exists() {
                        continue;
                    }

                    match std::fs::metadata(path) {
                        Ok(metadata) => {
                            if let Ok(current_mtime) = metadata.modified() {
                                let mut mtimes = file_mtimes.lock().unwrap();

                                if let Some(last_mtime) = mtimes.get(path) {
                                    if current_mtime > *last_mtime {
                                        // 文件已修改，更新时间戳
                                        mtimes.insert(path.clone(), current_mtime);
                                        config_changed = true;
                                    }
                                } else {
                                    // 新文件，添加到跟踪列表
                                    mtimes.insert(path.clone(), current_mtime);
                                    config_changed = true;
                                }
                            }
                        },
                        Err(_) => continue,
                    }
                }

                // 如果配置文件已更改，重新加载配置
                if config_changed {
                    // 使用相同的构建器重新构建配置
                    match builder.build() {
                        Ok(new_config) => {
                            // 保存旧配置以通知观察者
                            let old_config = {
                                let config_read = config.read().unwrap();
                                config_read.clone()
                            };

                            // 更新配置
                            {
                                let mut config_write = config.write().unwrap();
                                *config_write = new_config.clone();
                            }

                            // 通知所有观察者
                            for observer in observers.iter() {
                                observer.on_config_changed(&old_config, &new_config);
                            }

                            tracing::info!("Configuration reloaded successfully");
                        },
                        Err(e) => {
                            tracing::error!("Failed to reload configuration: {}", e);
                        }
                    }
                }

                // 等待下一次检查
                thread::sleep(check_interval);
            }
        });

        ConfigWatcherHandle {
            thread_handle: Some(thread_handle),
            running,
            config,
        }
    }
}

// 提供对监控器的控制
pub struct ConfigWatcherHandle {
    thread_handle: Option<thread::JoinHandle<()>>,
    running: Arc<RwLock<bool>>,
    config: Arc<RwLock<AppConfig>>,
}

impl ConfigWatcherHandle {
    // 停止监控
    pub fn stop(mut self) -> Result<(), ConfigError> {
        {
            let mut running = self.running.write().unwrap();
            *running = false;
        }

        if let Some(handle) = self.thread_handle.take() {
            handle.join().map_err(|_| {
                ConfigError::Other("Failed to join watcher thread".to_string())
            })?;
        }

        Ok(())
    }

    // 获取当前配置的副本
    pub fn get_config(&self) -> AppConfig {
        self.config.read().unwrap().clone()
    }

    // 获取当前配置的引用
    pub fn config_ref(&self) -> Arc<RwLock<AppConfig>> {
        Arc::clone(&self.config)
    }
}

// 示例观察者实现
pub struct LoggingObserver;

impl ConfigChangeObserver for LoggingObserver {
    fn on_config_changed(&self, old_config: &AppConfig, new_config: &AppConfig) {
        tracing::info!(
            "Configuration changed - Service: {} -> {}",
            old_config.service_name(),
            new_config.service_name()
        );

        // 检测环境变化
        if old_config.service.environment != new_config.service.environment {
            tracing::warn!(
                "Environment changed: {} -> {}",
                old_config.service.environment,
                new_config.service.environment
            );
        }

        // 检测数据库配置变化
        if old_config.database != new_config.database {
            tracing::warn!("Database configuration changed");
        }
    }
}
