mod config;
mod windows_singleton;

use clap::Parser;
use log::{debug, error, info, LevelFilter};
use rdev::{grab, simulate, Event, EventType, Key};
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

    #[arg(short, long, help = "Write log to file (keyremap.log)")]
    logfile: bool,
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

    let config_path = if let Some(config_path) = args.config {
        config_path
    } else {
        exe_dir.join("keyremap.toml")
    };

    info!("Loading config: {:?}", config_path);

    // 读取配置文件
    let config = load_config(&config_path).unwrap();
    
    #[cfg(target_os = "windows")]
    if !windows_singleton::ensure_single_instance(MUTEX_NAME) {
        return;
    }

    info!("=== KeyRemap start ===");

    info!("Listening for key mappings:");
    for mapping in &config.key_mappings {
        if mapping.enable {
            info!("- {}", mapping.name);
        };
    }
    info!("Press Ctrl+C to exit\n");

    key_handle_loop(config);
}

fn load_config(path: &PathBuf) -> Option<Config> {
    let config = match fs::read_to_string(path) {
        Ok(content) => match toml::from_str::<Config>(&content) {
            Ok(config) => {
                debug!("Configuration content: {:?}", config);
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

fn key_handle_loop(config: Config) {
    if let Err(error) = grab(move |event: Event| -> Option<Event> {
        match event.event_type {
            EventType::MouseMove { .. } => {}
            EventType::KeyPress(_) | EventType::KeyRelease(_) => {
                debug!(
                    "Captured key event: {:?} | Time: {:?}",
                    event.event_type, event.time
                );
            }
            EventType::ButtonPress(_) | EventType::ButtonRelease(_) => {
                debug!(
                    "Captured button event: {:?}, | Time: {:?}",
                    event.event_type, event.time
                );
            }
            _ => debug!(
                "Captured other event: {:?} | Time: {:?}",
                event.event_type, event.time
            ),
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
            if let Some(combination) = mapping.to.get_combination() {
                debug!("Simulate key combination: {:#?}", combination);
                // 按顺序按下所有键
                for key in &combination {
                    simulate_key(*key, true);
                }
                thread::sleep(Duration::from_millis(20));
                // 按相反顺序释放按键
                for key in combination.iter().rev() {
                    simulate_key(*key, false);
                }
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
            if mapping.to.get_combination().is_none() {
                if let Some(key) = mapping.to.key {
                    simulate_key(key, false);
                }
                true
            } else {
                false
            }
        }
        _ => false,
    }
}
