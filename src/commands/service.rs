use std::ffi::OsString;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use futures::executor::block_on;
use log::error;
use windows_service::service::{
    ServiceAccess, ServiceControl, ServiceControlAccept, ServiceErrorControl, ServiceExitCode,
    ServiceInfo, ServiceStartType, ServiceState, ServiceStatus, ServiceType,
};
use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};
use windows_service::{define_windows_service, service_dispatcher};

use duck::DuckResult;

///////////////////////////////////////////////////////////
// Constants

const SERVICE_EXECUTABLE: &str = "duck.exe";
const SERVICE_NAME: &str = "Duck Service";
const SERVICE_DISPLAY_NAME: &str = "Duck Service";
const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

///////////////////////////////////////////////////////////
// Installation

pub fn install() -> DuckResult<()> {
    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_info = ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from(SERVICE_DISPLAY_NAME),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::OnDemand,
        error_control: ServiceErrorControl::Normal,
        executable_path: std::env::current_exe()?.with_file_name(SERVICE_EXECUTABLE),
        launch_arguments: vec![OsString::from("-f"), OsString::from("service")],
        dependencies: vec![],
        account_name: None, // run as System
        account_password: None,
    };

    service_manager.create_service(&service_info, ServiceAccess::empty())?;

    Ok(())
}

///////////////////////////////////////////////////////////
// Uninstallation

pub fn uninstall() -> DuckResult<()> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE;
    let service = service_manager.open_service(SERVICE_NAME, service_access)?;

    let service_status = service.query_status()?;
    if service_status.current_state != ServiceState::Stopped {
        service.stop()?;
        // Wait for service to stop
        thread::sleep(Duration::from_secs(1));
    }

    service.delete()?;

    Ok(())
}

///////////////////////////////////////////////////////////
// Running

pub fn start() -> DuckResult<()> {
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
    Ok(())
}

// Generate the windows service boilerplate.
// The boilerplate contains the low-level service entry function (ffi_service_main) that parses
// incoming service arguments into Vec<OsString> and passes them to user defined service
// entry (duck_service_main).
define_windows_service!(ffi_service_main, duck_service_main);

pub fn duck_service_main(_arguments: Vec<OsString>) {
    if let Err(e) = run_service() {
        error!(
            "An error occured while running Duck as a Windows service: {}",
            e
        )
    }
}

pub fn run_service() -> DuckResult<()> {
    // Create a channel to be able to poll a stop event from the service worker loop.
    let (shutdown_tx, shutdown_rx) = mpsc::channel();

    // Define system service event handler that will be receiving service events.
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            // Notifies a service to report its current status information to the service
            // control manager. Always return NoError even if not implemented.
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,

            // Handle stop
            ServiceControl::Stop => {
                shutdown_tx.send(()).unwrap();
                ServiceControlHandlerResult::NoError
            }

            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // Register system service event handler.
    // The returned status handle should be used to report service status changes to the system.
    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    // Tell the system that service is running
    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::from_secs(10),
    })?;

    // Start Duck server.
    let configuration = std::env::current_exe()?.with_file_name("config.json");
    let handle = duck::run(configuration, None)?;

    // Wait for exit
    loop {
        // Poll shutdown event.
        match shutdown_rx.recv_timeout(Duration::from_secs(1)) {
            Ok(_) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                block_on(handle.stop())?;
                break;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => (),
        };
    }

    // Tell the system that service has stopped.
    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::from_secs(10),
    })?;

    Ok(())
}
