use crate::child::ChildSdvxIo;
use crate::error::Error;
use crate::glue::{log_formatter_t, thread_create_t, thread_destroy_t, thread_join_t};
use crate::logger::BT5Logger;
use sdvxio_pipe_proto::{ChildToParent, ParentToChild, Receiver, Sender};
use std::sync::Mutex;

mod child;
mod error;
mod logger;

mod glue {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(unused)]

    include!(concat!(env!("OUT_DIR"), "/glue.rs"));
}

static CHILD_SDVXIO: std::sync::LazyLock<Mutex<Option<ChildSdvxIo>>> =
    std::sync::LazyLock::new(|| Mutex::new(None));

fn with_child_sdvxio<T, E>(func: impl FnOnce(&mut ChildSdvxIo) -> Result<T, E>) -> Result<T, E> {
    let mut child_sdvxio = CHILD_SDVXIO.lock().expect("failed to lock child sdvxio");
    let child_sdvxio = child_sdvxio
        .as_mut()
        .expect("child SDVXIO is not initialized");
    func(&mut *child_sdvxio)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sdvx_io_set_loggers(
    misc: log_formatter_t,
    info: log_formatter_t,
    warning: log_formatter_t,
    fatal: log_formatter_t,
) {
    log::set_boxed_logger(Box::new(BT5Logger {
        misc,
        info,
        warning,
        fatal,
    }))
    .map(|_| log::set_max_level(log::LevelFilter::Info))
    .unwrap();
    panic_log::initialize_hook(panic_log::Configuration::default());
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sdvx_io_init(
    _thread_create: thread_create_t,
    _thread_join: thread_join_t,
    _thread_destroy: thread_destroy_t,
) -> bool {
    log::trace!("sdvx_io_init called");

    // Spawn the pipe program process, located in the pipe subdirectory
    let mut child = std::process::Command::new("pipe/sdvxio-pipe-program.exe")
        .current_dir(std::env::current_dir().unwrap().join("pipe"))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("Failed to start sdvxio-pipe-program.exe");

    let tx = Sender::new(child.stdin.take().unwrap());
    let rx = Receiver::new(child.stdout.take().unwrap());

    log::info!("Child sdvxio process started");

    let mut child_sdvxio = CHILD_SDVXIO.lock().expect("failed to lock child sdvxio");
    let old = child_sdvxio.replace(ChildSdvxIo { child, tx, rx });
    if let Some(mut old) = old {
        log::warn!("sdvxio was already initialized, terminating old process");
        let _ = old.child.kill();
    }
    drop(child_sdvxio);

    let success = with_child_sdvxio(|child| match child.request(ParentToChild::InitRequest)? {
        ChildToParent::InitResponse(value) => Ok(value),
        _ => Err(Error::WrongResponseType),
    })
    .unwrap_or_else(|err| {
        log::error!("Failed to initialize child sdvxio: {:?}", err);
        false
    });

    if success {
        log::info!("sdvxio initialized successfully");
    }

    success
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sdvx_io_fini() {
    log::trace!("sdvx_io_fini called");

    with_child_sdvxio(
        |child| match child.request(ParentToChild::FinalizeRequest)? {
            ChildToParent::FinalizeResponse => Ok(()),
            _ => Err(Error::WrongResponseType),
        },
    )
    .unwrap_or_else(|err| {
        log::error!("Failed to finalize child sdvxio: {:?}", err);
    });

    let mut child_sdvxio = CHILD_SDVXIO.lock().expect("failed to lock child sdvxio");
    if let Some(mut child) = child_sdvxio.take() {
        let _ = child.child.kill();
        log::info!("sdvxio finalized and child process terminated");
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sdvx_io_set_gpio_lights(gpio_lights: u32) {
    log::trace!("sdvx_io_set_gpio_lights called");

    with_child_sdvxio(|child| {
        match child.request(ParentToChild::SetGpioLightsRequest(gpio_lights))? {
            ChildToParent::SetGpioLightsResponse => Ok(()),
            _ => Err(Error::WrongResponseType),
        }
    })
    .unwrap_or_else(|err| {
        log::error!("Failed to set GPIO lights on child sdvxio: {:?}", err);
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sdvx_io_set_pwm_light(light_no: u8, intensity: u8) {
    log::trace!("sdvx_io_set_pwm_light called");

    with_child_sdvxio(|child| {
        match child.request(ParentToChild::SetPwmLightRequest {
            light_no,
            intensity,
        })? {
            ChildToParent::SetPwmLightResponse => Ok(()),
            _ => Err(Error::WrongResponseType),
        }
    })
    .unwrap_or_else(|err| {
        log::error!("Failed to set PWM light on child sdvxio: {:?}", err);
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sdvx_io_write_output() -> bool {
    log::trace!("sdvx_io_write_output called");

    with_child_sdvxio(
        |child| match child.request(ParentToChild::WriteOutputRequest)? {
            ChildToParent::WriteOutputResponse(value) => Ok(value),
            _ => Err(Error::WrongResponseType),
        },
    )
    .unwrap_or_else(|err| {
        log::error!("Failed to write output to child sdvxio: {:?}", err);
        false
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sdvx_io_read_input() -> bool {
    log::trace!("sdvx_io_read_input called");

    with_child_sdvxio(
        |child| match child.request(ParentToChild::ReadInputRequest)? {
            ChildToParent::ReadInputResponse(success) => Ok(success),
            _ => Err(Error::WrongResponseType),
        },
    )
    .unwrap_or_else(|err| {
        log::error!("Failed to read input from child sdvxio: {:?}", err);
        false
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sdvx_io_get_input_gpio_sys() -> u8 {
    log::trace!("sdvx_io_get_input_gpio_sys called");

    with_child_sdvxio(
        |child| match child.request(ParentToChild::GetInputGpioSysRequest)? {
            ChildToParent::GetInputGpioSysResponse(value) => Ok(value),
            _ => Err(Error::WrongResponseType),
        },
    )
    .unwrap_or_else(|err| {
        log::error!("Failed to get input GPIO sys from child sdvxio: {:?}", err);
        0
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sdvx_io_get_input_gpio(gpio_bank: u8) -> u16 {
    log::trace!("sdvx_io_get_input_gpio called");

    with_child_sdvxio(|child| {
        match child.request(ParentToChild::GetInputGpioRequest(gpio_bank))? {
            ChildToParent::GetInputGpioResponse(value) => Ok(value),
            _ => Err(Error::WrongResponseType),
        }
    })
    .unwrap_or_else(|err| {
        log::error!("Failed to get input GPIO from child sdvxio: {:?}", err);
        0
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sdvx_io_get_spinner_pos(spinner_no: u8) -> u16 {
    log::trace!("sdvx_io_get_spinner_pos called");

    with_child_sdvxio(|child| {
        match child.request(ParentToChild::GetSpinnerPosRequest(spinner_no))? {
            ChildToParent::GetSpinnerPosResponse(value) => Ok(value),
            _ => Err(Error::WrongResponseType),
        }
    })
    .unwrap_or_else(|err| {
        log::error!("Failed to get spinner pos from child sdvxio: {:?}", err);
        0
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sdvx_io_set_amp_volume(primary: u8, headphone: u8, subwoofer: u8) -> bool {
    log::trace!("sdvx_io_set_amp_volume called");

    with_child_sdvxio(|child| {
        match child.request(ParentToChild::SetAmpVolumeRequest {
            primary,
            headphone,
            subwoofer,
        })? {
            ChildToParent::SetAmpVolumeResponse(value) => Ok(value),
            _ => Err(Error::WrongResponseType),
        }
    })
    .unwrap_or_else(|err| {
        log::error!("Failed to set amp volume on child sdvxio: {:?}", err);
        false
    })
}
