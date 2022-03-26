//! Mac level commands

use modular_bitfield::prelude::*;

/// Either sent as a FRMPayload with FPort = 0 or piggybacked in the FOpts field.
/// NOTE: piggybacked = always unencrypted, as FRMPayload = always encrypted
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub enum MacCommandCid {
    /// LinkCheckReq, LinkCheckAns
    LinkCheck = 0x02,
    /// LinkADRReq, LinkADRAns
    LinkAdr = 0x03,

    DutyCycle = 0x04,
    RxParamSetup = 0x05,
    DevStatus = 0x06,
    NewChannel = 0x07,
    RxTimingSetup = 0x08,
    TxParamSetup = 0x09,
    DlChannel = 0x0A,
    DeviceTime = 0x0D,
    // TODO: 0x10..=0x1F: Class B Commands
    // TODO: 0x20..=0x2F: Class C commands
    // TODO: 0x80..=0xFF: Proprietary network command extensions
}

#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct LinkCheck {
    pub margin: u8,
    pub gw_count: u8,
}

#[bitfield]
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct LinkAdrReq {
    pub data_rate: B4,
    pub tx_power: B4,
    pub ch_mask: u16,
    pub rfu: bool,
    pub channel_mask_ctrl: B3,
    pub nb_trans: B4,
}

#[bitfield]
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct LinkAdrAns {
    pub rfu: B5,
    pub power_ack: bool,
    pub data_rate_ack: bool,
    pub channel_mask_ack: bool,
}

#[bitfield]
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct DutyCycleReq {
    pub rfu: B4,
    pub max_duty_cycle: B4,
}

#[bitfield]
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct RxParamSetupReq {
    pub rfu: bool,
    pub rx1_data_rate_offset: B3,
    pub rx2_data_rate: B4,
}

#[bitfield]
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct RxParamSetupAns {
    pub rfu: B5,
    pub rx1_data_rate_offset_ack: bool,
    pub rx2_data_rate_ack: bool,
    pub channel_ack: bool,
}

#[bitfield]
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct DevStatusAns {
    pub battery: u8,
    pub rfu: B3,
    pub snr: B5,
}

#[bitfield]
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct NewChannelReq {
    pub channel_index: u8,
    pub frequency: B24,
    pub max_data_rate: B4,
    pub min_data_rate: B4,
}

#[bitfield]
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct NewChannelAns {
    pub rfu: B6,
    pub data_rate_range_ok: bool,
    pub channel_frequency_ok: bool,
}

#[bitfield]
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct DlChannelReq {
    pub channel_index: u8,
    pub frequency: B24,
}

#[bitfield]
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct DlChannelAns {
    pub rfu: B6,
    pub uplink_frequency_exists: bool,
    pub channel_frequency_ok: bool,
}

#[bitfield]
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct RxTimingSetupReq {
    pub rfu: B5,
    pub delay_seconds: B3,
}

#[bitfield]
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct TxParamSetup {
    pub rfu: B2,
    pub downlink_dwell_time: bool,
    pub uplink_dwell_time: bool,
    pub max_eirp: B4,
}

#[bitfield]
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct DeviceTimeAns {
    pub seconds_since_epoch: u32,
    pub fraction_seconds: u8,
}
