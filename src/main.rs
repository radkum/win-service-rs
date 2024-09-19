mod action;

use std::ffi::OsString;

use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
};

use crate::action::Action;

fn main() -> Result<(), windows_service::Error> {
    // Register generated `ffi_service_main` with the system and start the service, blocking
    // this thread until the service is stopped.
    let _ = windebug_logger::init();
    log::debug!("main");
    service_dispatcher::start("r", ffi_service_main)?;

    Ok(())
}

define_windows_service!(ffi_service_main, my_service_main);
fn my_service_main(arguments: Vec<OsString>) {
    log::debug!("my_service_main");
    if let Err(e) = run_service(arguments) {
        log::debug!("{e}");
    }
}

fn run_service(_arguments: Vec<OsString>) -> windows_service::Result<()> {
    let (sender, receiver) = std::sync::mpsc::channel::<Action>();

    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop => {
                sender.send(Action::Close).unwrap();
                ServiceControlHandlerResult::NoError
            },
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // Register system service event handler
    let status_handle = service_control_handler::register("radek", event_handler)?;

    let next_status = service_status(ServiceState::Running, ServiceControlAccept::STOP);

    // Tell the system that the service is running now
    status_handle.set_service_status(next_status)?;

    let mut i = 0;
    loop {
        log::debug!("loop {i}");
        i += 1;
        match receiver.recv_timeout(std::time::Duration::from_secs(1)) {
            Ok(action) => {
                match action {
                    Action::Close => {
                        status_handle.set_service_status(service_status(
                            ServiceState::StopPending,
                            ServiceControlAccept::empty(),
                        ))?;
                    },
                }
                break;
            },
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                log::debug!("disconnected");
                //todo
            },
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => (),
        }
    }
    log::debug!("Exited main service loop");
    status_handle
        .set_service_status(service_status(ServiceState::Stopped, ServiceControlAccept::empty()))?;

    Ok(())
}

fn service_status(
    current_state: ServiceState,
    controls_accepted: ServiceControlAccept,
) -> ServiceStatus {
    ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state,
        controls_accepted,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: std::time::Duration::default(),
        process_id: None,
    }
}
