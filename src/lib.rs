//! LoRaWAN implimentation based on the LoRaWAN L2 1.0.4 Specification
//!
//! Supports `no_std`.
#![no_std]

use modular_bitfield::prelude::*;

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

// Class A:
// Following each uplink transmission, the end-device SHALL open one or two receive windows
// (RX1 and RX2); if no packet destined for the end-device is received in RX1, it SHALL open
// RX2. The receive windows start times are defined using the end of the transmission as a
// reference, see Figure 2.

//
// PHYPayload:
// ```norust
//  1   | 7..M         | 4
// MHDR | MACPayload   | MIC
//
// MHDR | Join-Request | MIC
// MHDR | Join-Accept  | MIC
// ```
//
// MACPayload:
// ```norust
// FHDR | FPort | FRMPayload
// ```
//
// FHDR:
// ```norust
// DevAddr | FCtrl | FCnt | FOpts
// ```
//
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct PhyPayload {}

/// MHDR
#[bitfield]
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct MacHeader {
    #[bits = 3]
    pub ftype: FrameType,
    pub rfu: B3,
    pub major: B2,
}

/// FType, 3 bits
#[repr(u8)]
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy, BitfieldSpecifier)]
pub enum FrameType {
    JoinRequest = 0b000,
    JoinAccept = 0b001,
    UnconfirmedDataUplink = 0b010,
    UnconfirmedDataDownlink = 0b011,
    ConfirmedDataUplink = 0b100,
    ConfirmedDataDownlink = 0b101,
    Rfu = 0b110,
    Proprietary = 0b111,
}

#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct MacPayload {
    pub dev_addr: [u8; 4],
    pub fctrl: u8,
    pub fcnt: u8,

    // FIXME: 0 to 15 bytes
    pub fopts: [u8; 15],
}

#[bitfield]
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct DownlinkFrameControl {
    pub adr: bool,
    pub rfu: bool,
    pub ack: bool,
    pub frame_pending: bool,
    pub frame_opts_len: B4,
}

#[bitfield]
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct UplinkFrameControl {
    pub adr: bool,
    pub adr_ack_req: bool,
    pub ack: bool,
    pub class_b: bool,
    pub frame_opts_len: B4,
}

#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub enum DataRate {
    Dr0,
    Dr1,
    Dr2,
    Dr3,
    Dr4,
    Dr8,
    Dr13,
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
            data_rate: DataRate::Dr0,
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
    addr: u32,
}
