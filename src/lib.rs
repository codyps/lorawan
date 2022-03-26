//! LoRaWAN implimentation based on the LoRaWAN L2 1.0.4 Specification
//!
//! Supports `no_std`.
#![no_std]

// "Classes" refers to device classes:
//  - Class A: Bi-directional end-devices
//    - each end-device's uplink transmission is followed by 2 short downlink receive windows
//    - uplink scheduled by the end-device
//    - lowest power end-device
//  - Class B: Bi-directional end-devices with scheduled recieve slots
//    - Class A rx windows, plus additional scheduled rx windows.
//    - Recvs time-scynchronized beacon from the gateway
//  - Class C: Bi-drectional end-devices with maximal receive slots
//    - continuously open receive windows, closed only when transmitting

pub mod parameters;
pub use parameters::*;

pub mod encode;
pub mod mac;

#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub enum DataRate {
    _0,
    _1,
    _2,
    _3,
    _4,
    _5,
    _6,
    _7,
    _8,
    _9,
    _10,
    _11,
    _12,
    _13,
    _14,
    _15,
}

#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub enum CodingRate {
    Cr1_3,
    Cr2_3,
    Cr4_5,
}

#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct BackoffDetails {
    pub data_rate: DataRate,
    pub adr_ack_req: bool,
    pub tx_power: (),
    pub nb_trans: u8,
    pub channel_mask_reset: bool,
}

// "4.3.1.1 Adaptive data-rate control in frame header (ADR, ADRACKReq in FCtrl)"
pub fn data_rate_backoff(params: &Parameters, adr_ack_cnt: u32) -> BackoffDetails {
    if adr_ack_cnt < params.adr_ack_limit {
        return BackoffDetails {
            data_rate: DataRate::_0,
            adr_ack_req: false,
            tx_power: (), // Max -9dBm
            nb_trans: 3,
            channel_mask_reset: false,
        };
    }

    panic!()
}

#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct Frequency {
    pub khz: u32,
}

impl Frequency {
    pub fn from_mhz(mhz: f32) -> Self {
        Self {
            khz: (mhz * 1000f32) as _,
        }
    }

    pub const fn from_khz(khz: u32) -> Self {
        Self { khz }
    }
}

#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct DevAddr {
    pub addr: u32,
}

#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct EndDevice {
    /// FCntUp: Incremented by an end-device when a data frame is transmitted to a Network Server (uplink).
    ///
    /// - Over the air activated devices (OOTA): set to 0 when a JoinAccept is succesfully processed
    /// - Activation by personalization (ABP): set to 0 by manufacturer. Never otherwise reset.
    ///   Must be persisted for lifetime of device.
    pub frame_count_uplink: u32,

    /// XXX
    pub adr_ack_cnt: u32,
}

/// Representation of the `Network Server` view of a device
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct Network {
    /// FCntDown: Incremented by a Network Server when a data frame is transmitted to an end-device (downlink)
    ///
    /// - Over the air activated devices (OOTA): set to 0 when a JoinAccept is succesfully processed
    pub frame_count_downlink: u32,
}

/// Data stored in end-device after activation
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct EndDeviceStorageActivation {
    pub dev_addr: DevAddr,
    pub network_session_key: (),
    pub application_session_key: (),
}

#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct EndDeviceStorage {
    pub activation: EndDeviceStorageActivation,

    /// Used for 6.2.5 Join-Request frame.
    pub dev_nonce: u16,
}

#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct JoinServer {
    /// "Join Server keeps track of the last DevNonce value used by the end-device and ignores
    /// Join-Requests if the DevNonce is not incremented
    pub dev_nonce: u16,
}
