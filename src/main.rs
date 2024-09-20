mod action;

use std::{
    ffi::{c_void, CString, OsString},
    ptr::null_mut,
};

use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType, UserEventCode,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
};
use windows_sys::Win32::{
    Foundation::{GetLastError, FALSE},
    System::Services::*,
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

const UNPROTECT: UserEventCode = unsafe { UserEventCode::from_unchecked(0x00000080) };

fn run_service(arguments: Vec<OsString>) -> windows_service::Result<()> {
    let (sender, receiver) = std::sync::mpsc::channel::<Action>();

    let mut service_name = String::new();

    if arguments.len() > 0 {
        if let Ok(str) = arguments[0].clone().into_string() {
            service_name = str
        }
    }

    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop => {
                sender.send(Action::Close).unwrap();
                ServiceControlHandlerResult::NoError
            },
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            ServiceControl::UserEvent(user_event_code) => match user_event_code {
                UNPROTECT => unprotect_current_service(service_name.as_str()),
                _ => ServiceControlHandlerResult::NotImplemented,
            },

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

fn unprotect_current_service(service_name: &str) -> ServiceControlHandlerResult {
    fn get_last_error() -> u32 {
        unsafe { GetLastError() }
    }

    // todo: give only necessary rights
    let h_manager = unsafe { OpenSCManagerA(null_mut(), null_mut(), SC_MANAGER_ALL_ACCESS) };
    if h_manager.is_null() {
        log::error!("Failed to call OpenSCManagerA: {}", get_last_error());
        return ServiceControlHandlerResult::Other(1);
    }

    let service_name = CString::new(service_name).unwrap();

    // todo: give only necessary rights. Maybe: SERVICE_USER_DEFINED_CONTROL
    let h_service =
        unsafe { OpenServiceA(h_manager, service_name.as_ptr() as *const u8, SERVICE_ALL_ACCESS) };
    if h_service.is_null() {
        log::error!("Failed to call OpenServiceA: {}", get_last_error());
        return ServiceControlHandlerResult::Other(1);
    }

    let info: SERVICE_LAUNCH_PROTECTED_INFO =
        SERVICE_LAUNCH_PROTECTED_INFO { dwLaunchProtected: SERVICE_LAUNCH_PROTECTED_NONE };
    let status = unsafe {
        ChangeServiceConfig2A(
            h_service,
            SERVICE_CONFIG_LAUNCH_PROTECTED,
            &info as *const SERVICE_LAUNCH_PROTECTED_INFO as *const c_void,
        )
    };

    if status == FALSE {
        log::error!("Failed to call ChangeServiceConfig2A: {}", get_last_error());
        return ServiceControlHandlerResult::Other(1);
    }

    ServiceControlHandlerResult::NoError
}
