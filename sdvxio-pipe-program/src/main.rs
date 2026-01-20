#![feature(c_variadic)]

use crate::bt5api::{FATAL, INFO, MISC, WARN};
use sdvxio_pipe_proto::{ChildToParent, Message, ParentToChild, Receiver, Sender};
use std::io::Stdout;

mod bt5api;
mod log;

fn main() {
    log::Logger::new().init();
    panic_log::initialize_hook(panic_log::Configuration::default());
    unsafe {
        bt5api::sdvx_io_set_loggers(
            Some(bt5api::log::<MISC>),
            Some(bt5api::log::<INFO>),
            Some(bt5api::log::<WARN>),
            Some(bt5api::log::<FATAL>),
        );
    };

    log::info!("Starting sdvxio-pipe-program");

    let mut tx = Sender::new(std::io::stdout());
    let mut rx = Receiver::<_, Message<ParentToChild>>::new(std::io::stdin());

    log::info!("Starting main loop");
    while let Ok(msg) = rx.recv() {
        handle_message(&mut tx, msg);
    }
}

fn handle_message(tx: &mut Sender<Stdout, Message<ChildToParent>>, msg: Message<ParentToChild>) {
    match msg.payload {
        ParentToChild::InitRequest => {
            let success = unsafe {
                bt5api::sdvx_io_init(
                    Some(bt5api::create_thread),
                    Some(bt5api::join_thread),
                    Some(bt5api::destroy_thread),
                )
            };
            tx.send(&msg.reply(ChildToParent::InitResponse(success)))
                .expect("failed to send response");
        }
        ParentToChild::FinalizeRequest => {
            unsafe { bt5api::sdvx_io_fini() };
            tx.send(&msg.reply(ChildToParent::FinalizeResponse))
                .expect("failed to send response");
            std::thread::sleep(std::time::Duration::from_secs(1));
            std::process::exit(0);
        }
        ParentToChild::SetGpioLightsRequest(lights) => {
            unsafe { bt5api::sdvx_io_set_gpio_lights(lights) };
            tx.send(&msg.reply(ChildToParent::SetGpioLightsResponse))
                .expect("failed to send response");
        }
        ParentToChild::SetPwmLightRequest {
            light_no,
            intensity,
        } => {
            unsafe { bt5api::sdvx_io_set_pwm_light(light_no, intensity) };
            tx.send(&msg.reply(ChildToParent::SetPwmLightResponse))
                .expect("failed to send response");
        }
        ParentToChild::WriteOutputRequest => {
            let result = unsafe { bt5api::sdvx_io_write_output() };
            tx.send(&msg.reply(ChildToParent::WriteOutputResponse(result)))
                .expect("failed to send response");
        }
        ParentToChild::ReadInputRequest => {
            let result = unsafe { bt5api::sdvx_io_read_input() };
            tx.send(&msg.reply(ChildToParent::ReadInputResponse(result)))
                .expect("failed to send response");
        }
        ParentToChild::GetInputGpioSysRequest => {
            let result = unsafe { bt5api::sdvx_io_get_input_gpio_sys() };
            tx.send(&msg.reply(ChildToParent::GetInputGpioSysResponse(result)))
                .expect("failed to send response");
        }
        ParentToChild::GetInputGpioRequest(bank) => {
            let result = unsafe { bt5api::sdvx_io_get_input_gpio(bank) };
            tx.send(&msg.reply(ChildToParent::GetInputGpioResponse(result)))
                .expect("failed to send response");
        }
        ParentToChild::GetSpinnerPosRequest(spinner) => {
            let result = unsafe { bt5api::sdvx_io_get_spinner_pos(spinner) };
            tx.send(&msg.reply(ChildToParent::GetSpinnerPosResponse(result)))
                .expect("failed to send response");
        }
        ParentToChild::SetAmpVolumeRequest {
            primary,
            headphone,
            subwoofer,
        } => {
            let result = unsafe { bt5api::sdvx_io_set_amp_volume(primary, headphone, subwoofer) };
            tx.send(&msg.reply(ChildToParent::SetAmpVolumeResponse(result)))
                .expect("failed to send response");
        }
    }
}
