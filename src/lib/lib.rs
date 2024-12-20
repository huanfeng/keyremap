mod config;
#[cfg(target_os = "windows")]
mod windows_console;
#[cfg(target_os = "windows")]
mod windows_singleton;

use clap::Parser;
use log::{debug, error, info, LevelFilter};
use rdev::{grab, listen, simulate, Event, EventType, Key};
use std::fs;
use std::path::PathBuf;
use std::{env, thread, time::Duration};

use config::{Config, KeyMapping};

static MUTEX_NAME: &str = "KeyremapSingleInstance";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, help = "Path to toml config file, default: keyremap.toml")]
    config: Option<PathBuf>,

    #[arg(short, action = clap::ArgAction::Count, help = "Set log level, ex: -v, -vv")]
    verbose: u8,

    #[arg(long, help = "Write log to file (keyremap.log)")]
    logfile: bool,

    #[arg(short, long, help = "Listen mode")]
    listen: bool,

    #[arg(long, help = "Dump config")]
    dump: bool,

    #[cfg(target_os = "windows")]
    #[arg(short, long, help = "Daemon mode")]
    daemon: bool,
}

pub fn remap_main() {
    let args = Args::parse();
    let exe_path = env::current_exe().unwrap();
    let exe_dir = exe_path.parent().unwrap();

    // 设置日志级别
    let log_level = match args.verbose {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    if args.logfile {
        let log_file = std::fs::File::create(exe_dir.join("keyremap.log")).unwrap();
        env_logger::Builder::new()
            .filter_level(log_level)
            .target(env_logger::Target::Pipe(Box::new(log_file)))
            .init();
    } else {
        env_logger::Builder::new().filter_level(log_level).init();
    }

    if args.listen {
        info!("=== KeyRemap LISTEN ===");
        info!("Press Ctrl+C to exit\n");
        key_listen_loop();
        return;
    }

    let config_path = if let Some(config_path) = args.config {
        config_path
    } else {
        exe_dir.join("keyremap.toml")
    };

    info!("Loading config: {:?}", config_path);

    // 读取配置文件
    let config = load_config(&config_path).unwrap();

    if args.dump {
        info!("Dumping config:");
        for mapping in &config.key_mappings {
            info!("Mapping: {}", mapping.name);
            info!("  enabled: {}", mapping.enable);
            info!("  from: {}", mapping.from);
            info!("  to: {}", mapping.to);
        }
        return;
    }

    #[cfg(target_os = "windows")]
    if !windows_singleton::ensure_single_instance(MUTEX_NAME) {
        return;
    }

    info!("=== KeyRemap START ===");

    info!("Listening for key mappings:");
    for mapping in &config.key_mappings {
        if mapping.enable {
            info!("- {}", mapping.name);
        };
    }
    info!("Press Ctrl+C to exit\n");

    #[cfg(target_os = "windows")]
    if args.daemon {
        windows_console::free_console();
    }

    key_handle_loop(config);
}

fn load_config(path: &PathBuf) -> Option<Config> {
    let config = match fs::read_to_string(path) {
        Ok(content) => match toml::from_str::<Config>(&content) {
            Ok(config) => {
                debug!("Config content: {:?}", config);
                config
            }
            Err(e) => {
                error!("Failed to parse config file: {}", e);
                return None;
            }
        },
        Err(e) => {
            error!("Failed to read config file: {}", e);
            return None;
        }
    };
    Some(config)
}

fn key_listen_loop() {
    if let Err(error) = listen(move |event: Event| -> () {
        match event.event_type {
            EventType::MouseMove { .. } => {}
            EventType::KeyPress(_) | EventType::KeyRelease(_) => {
                info!("Key: {:?}", event.event_type);
            }
            EventType::ButtonPress(_) | EventType::ButtonRelease(_) => {
                info!("MouseButton: {:?}", event.event_type);
            }
            _ => debug!("Other: {:?} | Time: {:?}", event.event_type, event.time),
        }
    }) {
        error!("Capture error: {:?}", error);
    }
}

fn key_handle_loop(config: Config) {
    if let Err(error) = grab(move |event: Event| -> Option<Event> {
        match event.event_type {
            EventType::MouseMove { .. } => {}
            EventType::KeyPress(_) | EventType::KeyRelease(_) => {
                debug!("Key: {:?} | Time: {:?}", event.event_type, event.time);
            }
            EventType::ButtonPress(_) | EventType::ButtonRelease(_) => {
                debug!(
                    "MouseButton: {:?}, | Time: {:?}",
                    event.event_type, event.time
                );
            }
            _ => debug!("Other: {:?} | Time: {:?}", event.event_type, event.time),
        }

        for mapping in &config.key_mappings {
            if !mapping.enable {
                continue;
            };
            if process_mapping(mapping, &event) {
                return None;
            }
        }
        Some(event)
    }) {
        error!("Capture error: {:?}", error);
    }
}

fn simulate_key(key: Key, is_press: bool) {
    let event_type = if is_press {
        debug!("  -> Simulate press: {:?}", key);
        EventType::KeyPress(key)
    } else {
        debug!("  -> Simulate release: {:?}", key);
        EventType::KeyRelease(key)
    };

    if let Err(e) = simulate(&event_type) {
        error!("Simulate failed! {:?}", e);
    }
}

fn simulate_combination(combination: Vec<Key>) {
    // 按顺序按下所有键
    for key in &combination {
        simulate_key(*key, true);
    }
    thread::sleep(Duration::from_millis(20));
    // 按相反顺序释放按键
    for key in combination.iter().rev() {
        simulate_key(*key, false);
    }
}

// 返回 true 表示事件已经被处理, 屏蔽按键
fn process_mapping(mapping: &KeyMapping, event: &Event) -> bool {
    // 检查输入事件是否匹配
    let matches_input = match event.event_type {
        EventType::KeyPress(key) | EventType::KeyRelease(key) => mapping.from.matches_key(key),
        EventType::ButtonPress(button) | EventType::ButtonRelease(button) => {
            mapping.from.matches_button(button)
        }
        _ => false,
    };
    if !matches_input {
        return false;
    }

    debug!("Matching item: {}", mapping.name);

    match event.event_type {
        EventType::KeyPress(_) | EventType::ButtonPress(_) => {
            // 处理按键按下事件
            if let Some(_) = mapping.to.get_combination() {
                debug!("Has combination ignore key, deal later...");
                true
            } else if let Some(key) = mapping.to.key {
                simulate_key(key, true);
                true
            } else {
                false
            }
        }
        EventType::KeyRelease(_) | EventType::ButtonRelease(_) => {
            // 如果是组合键，释放事件已经在按下时处理过了
            if let Some(combination) = mapping.to.get_combination() {
                simulate_combination(combination);
                true
            } else if let Some(key) = mapping.to.key {
                simulate_key(key, false);
                true
            } else {
                false
            }
        }
        _ => false,
    }
}
