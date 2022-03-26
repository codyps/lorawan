use get_move::{Chain, Get};

use super::*;
use core::time::Duration;

mod eu863;
pub use eu863::Eu868;

mod us915;
pub use us915::Us915;

/// Parameters with recommended values consistent across all regions
///
/// If these parameters differ from the recommendations, those parameters shall be communicated to
/// the network server using an out-of-band channel during the end-device commissioning process.
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct Parameters {
    /// Default: 1s
    pub receive_delay1: Duration,

    /// Default: 2s
    pub receive_delay2: Duration,

    /// Default: 0 (table index)
    pub rx1_dr_offset: usize,

    /// Default: 5s
    pub join_accept_delay1: Duration,

    /// Default: 6s
    pub join_accept_delay2: Duration,

    /// Default: 16384
    pub max_fcnt_gap: u32,

    /// Default: 64
    pub adr_ack_limit: u32,

    /// Default: 32
    pub adr_ack_delay: u32,

    /// Default: 1s
    pub retransmit_timeout_fixed: Duration,

    /// Default: 2s
    pub retransmit_timeout_random: Duration,

    /// Default: 0
    pub downlink_dwell_time: Duration,

    /// Default: 7 (2**7 = 128s)
    pub ping_slot_periodicity: u8,

    /// Default: 8s
    pub class_b_resp_timeout: Duration,

    /// Default: 8s
    pub class_c_resp_timeout: Duration,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            receive_delay1: Duration::from_secs(1),
            receive_delay2: Duration::from_secs(2),

            rx1_dr_offset: 0,

            join_accept_delay1: Duration::from_secs(5),
            join_accept_delay2: Duration::from_secs(6),

            max_fcnt_gap: 16384,

            adr_ack_limit: 64,
            adr_ack_delay: 32,

            retransmit_timeout_fixed: Duration::from_secs(1),
            retransmit_timeout_random: Duration::from_secs(2),

            downlink_dwell_time: Duration::from_secs(0),
            ping_slot_periodicity: 7,
            class_b_resp_timeout: Duration::from_secs(8),
            class_c_resp_timeout: Duration::from_secs(8),
        }
    }
}

/// Parameters that are changable via MAC commands in the LoRaWAN specification
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct MutableParameters {}

/// Parameters that vary dependent on region
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct RegionalParameters {
    /// country specific!
    pub uplink_dwell_time: Duration,

    /// BEACON DR defined for each regional band
    pub ping_slot_datarate: DataRate,

    /// defined in each regional band
    pub ping_slot_channel: u8,
}

/// An index into a per-band table of modulation modes
///
/// This index is returned by various functions instead of specifying the modulation directly. Only
/// values 0 to 15 are valid.
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Copy, Clone)]
pub struct DataRate(u8);

/// RP002-1.0.3
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum BandId {
    Eu868 = 1,
    US915 = 2,
    CN779 = 3,
    EU433 = 4,
    AU915 = 5,
    CN470 = 6,
    AS923 = 7,
    AS923_2 = 8,
    AS923_3 = 9,
    KR920 = 10,
    IN865 = 11,
    RU864 = 12,
    AS923_4 = 13,
}

trait Band {
    type UpstreamChannels: Get<Output = ChannelDetails>;
    type DownstreamChannels: Get<Output = ChannelDetails>;

    fn upstream_channels(&self) -> &Self::UpstreamChannels;
    fn downstream_channels(&self) -> &Self::DownstreamChannels;

    fn data_rates(&self) -> &[Modulation];

    /// Given the current data rate,  what is the next data rate to use during backoff
    ///
    /// This applies when the device is using Adaptive Data Rate mode
    ///
    /// Returns `None` if there is no specified backoff data rate, normally because the input data
    /// rate was out of range.
    fn backoff_data_rate(&self, dr_current: DataRate) -> Option<DataRate>;

    /// OPTIONAL CFlist that can be appened to the JoinAccept message is of this type if present
    fn cflist_type(&self) -> CflistType;

    fn maximum_payload_size(data_rate: u8) -> Option<u8>;

    /// Maximum payload size if the end-device will never operate under a repeater
    fn maximum_payload_size_absent_repeaters(data_rate: u8) -> Option<u8>;

    /// Provide defaults for beacons
    fn beacon_settings(&self) -> &BeaconSettings;

    fn rx1_recv_channel(transmit_channel: u8) -> u8;

    fn rx1_window_data_rate(upstream_datarate: DataRate, rx1_dr_offset: u8) -> Option<DataRate>;

    fn rx2_window_details(&self) -> (Frequency, DataRate);
}

#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct Channels<U, D> {
    upstream: U,
    downstream: D,
}

#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum CflistType {
    /// This list is a series of 3 byte fields, each representing a frequency in 100Hz units
    Specific = 0,

    /// This is a list of 2 byte fields (7 of them). They are masks over the channels defined by
    /// the in-use band
    Mask = 1,
}

#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub enum Polarity {
    Normal,
    Inverted,
}

/// A chunk of adjacent channels with some associated data
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct ChannelPlan {
    pub first_channel: Frequency,
    pub channel_step: Frequency,
    pub count: usize,

    pub bandwidth: Frequency,
    pub coding_rate: Option<CodingRate>,
    pub data_rate_min: DataRate,
    pub data_rate_max: DataRate,
}

impl<'a> IntoIterator for &'a ChannelPlan {
    type IntoIter = get_move::Iter<'a, ChannelPlan>;
    type Item = ChannelDetails;

    fn into_iter(self) -> Self::IntoIter {
        get_move::Get::iter(self)
    }
}

impl get_move::Get for ChannelPlan {
    type Output = ChannelDetails;
    fn get_move(&self, index: usize) -> Option<Self::Output> {
        todo!()
    }

    fn len(&self) -> usize {
        self.count
    }
}

#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct ChannelDetails {
    pub frequency: Frequency,
    pub bandwidth: Frequency,

    // FIXME: this specification of valid channel configuration isn't flexible enough
    pub data_rate_min: DataRate,
    pub data_rate_max: DataRate,
    // FIXME: some bands specify this for channels and others do not
    pub coding_rate: Option<CodingRate>,
}

#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub enum Modulation {
    Lora {
        sf: u8,
        bw: Frequency,
    },
    Fsk {
        /// kbps
        rate: u32,
    },
    LrFhss {
        coding_rate: CodingRate,
        bandwidth: Frequency,
    },
    Rfu,
}

#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct BeaconSettings {
    pub dr: DataRate,
    pub cr: CodingRate,
    pub polarity: Polarity,
    pub channels: ChannelSpec,
}

#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub enum ChannelSpec {
    /// Pick one of the downstream channels based on `channel_for_beacon()`
    AllDownstream,
    /// Use this exact one frequency as the beacon default broadcast frequency
    ///
    // TODO: pingSlot?
    One(Frequency),
}

/// Used when `BeaconSettings::channels` is set to `AllDownstream`
pub fn channel_for_beacon(beacon_time: u32, beacon_period: u32) -> u8 {
    ((beacon_time / beacon_period) % 8) as u8
}
