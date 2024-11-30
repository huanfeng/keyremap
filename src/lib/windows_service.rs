use log::{debug, error, info};
use std::path::PathBuf;
use std::time::Duration;
use std::{ffi::OsString, thread};
use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
};

const SERVICE_NAME: &str = "KeyRemapService";
const SERVICE_DISPLAY_NAME: &str = "Key Remap Service";
const SERVICE_DESCRIPTION: &str = "A service for keyboard remapping functionality";

define_windows_service!(ffi_service_main, service_main);

pub fn run_service() -> windows_service::Result<()> {
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
    Ok(())
}

fn service_main(arguments: Vec<OsString>) {
    if let Err(e) = run_service_main(arguments) {
        // Handle error, possibly log it
        info!("Service failed: {}", e);
    }
}

fn run_service_main(arguments: Vec<OsString>) -> windows_service::Result<()> {

    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        debug!("Received control event: {:?}", control_event);
        match control_event {
            ServiceControl::Stop => {
                // Handle stopping the service
                
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    // 加载配置文件
    let config_path = PathBuf::from(
        "D:\\Develop\\workspace\\rust_project\\keyremap\\target\\debug\\keyremap.toml",
    );
    match super::load_config(&config_path) {
        Some(config) => {
            info!("Service started with config from: {:?}", config_path);
            // 启动键盘映射功能
            thread::spawn(move || {
                super::key_handle_loop(config);
            });
        }
        None => {
            error!("Failed to load config from: {:?}", config_path);
            status_handle.set_service_status(ServiceStatus {
                service_type: ServiceType::OWN_PROCESS,
                current_state: ServiceState::Stopped,
                controls_accepted: ServiceControlAccept::empty(),
                exit_code: ServiceExitCode::ServiceSpecific(1),
                checkpoint: 0,
                wait_hint: Duration::default(),
                process_id: None,
            })?;
        }
    }

    Ok(())
}

pub fn install_service() -> windows_service::Result<()> {
    use windows_service::{
        service::{ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceStartType},
        service_manager::{ServiceManager, ServiceManagerAccess},
    };

    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_binary_path = std::env::current_exe().unwrap();

    let service_info = ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from(SERVICE_DISPLAY_NAME),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: service_binary_path,
        launch_arguments: vec![OsString::from("--service")],
        dependencies: vec![],
        account_name: None,
        account_password: None,
    };

    let service = service_manager.create_service(&service_info, ServiceAccess::CHANGE_CONFIG)?;
    service.set_description(SERVICE_DESCRIPTION)?;

    Ok(())
}

pub fn uninstall_service() -> windows_service::Result<()> {
    use windows_service::{
        service::ServiceAccess,
        service_manager::{ServiceManager, ServiceManagerAccess},
    };

    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE;
    let service = service_manager.open_service(SERVICE_NAME, service_access)?;

    // Stop the service if it's running
    let service_status = service.query_status()?;
    if service_status.current_state != ServiceState::Stopped {
        service.stop()?;
        // Wait for service to stop
        while service.query_status()?.current_state != ServiceState::Stopped {
            std::thread::sleep(Duration::from_secs(1));
        }
    }

    // Delete the service
    service.delete()?;
    Ok(())
}
