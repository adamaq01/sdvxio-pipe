mod pipe;
pub use pipe::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Message<T> {
    pub id: u32,
    pub payload: T,
}

#[derive(Serialize, Deserialize)]
pub enum ChildToParent {
    InitResponse(bool),
    WriteOutputResponse(bool),
    ReadInputResponse(bool),
    GetInputGpioSysResponse(u8),
    GetInputGpioResponse(u16),
    GetSpinnerPosResponse(u16),
    SetAmpVolumeResponse(bool),
    SetPwmLightResponse,
    SetGpioLightsResponse,
    FinalizeResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ParentToChild {
    InitRequest,
    WriteOutputRequest,
    ReadInputRequest,
    GetInputGpioSysRequest,
    GetInputGpioRequest(u8),
    GetSpinnerPosRequest(u8),
    SetAmpVolumeRequest {
        primary: u8,
        headphone: u8,
        subwoofer: u8,
    },
    SetPwmLightRequest {
        light_no: u8,
        intensity: u8,
    },
    SetGpioLightsRequest(u32),
    FinalizeRequest,
}

impl<T> Message<T> {
    pub fn new(payload: T) -> Self {
        static NEXT_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        Self { id, payload }
    }

    pub fn with_id(id: u32, payload: T) -> Self {
        Self { id, payload }
    }

    pub fn reply<U>(&self, payload: U) -> Message<U> {
        Message {
            id: self.id,
            payload,
        }
    }
}
