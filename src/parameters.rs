use get_move::{Chain, Get};

use super::*;
use core::time::Duration;

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

impl BandId {}

trait Band {
    type UpstreamChannels: Get<Output = ChannelDetails>;
    type DownstreamChannels: Get<Output = ChannelDetails>;

    fn upstream_channels(&self) -> &Self::UpstreamChannels;
    fn downstream_channels(&self) -> &Self::DownstreamChannels;

    fn data_rates(&self) -> &[Modulation];

    /// Given the current data rate,  what is the next data rate to use during backoff
    ///
    /// This applies when the device is using Adaptive Data Rate mode
    fn backoff_data_rate(&self, dr_current: DataRate) -> DataRate;

    /// OPTIONAL CFlist that can be appened to the JoinAccept message is of this type if present
    fn cflist_type(&self) -> CflistType;

    fn maximum_payload_size(data_rate: u8) -> Option<u8>;

    /// Maximum payload size if the end-device will never operate under a repeater
    fn maximum_payload_size_absent_repeaters(data_rate: u8) -> Option<u8>;
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

/// US902-928 MHz ISM Band
///
/// As defined by RP002-1.0.3, 2.5 (line 531)
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct Us915;

impl Band for Us915 {
    type UpstreamChannels = Chain<ChannelPlan, ChannelPlan>;
    fn upstream_channels(&self) -> &Self::UpstreamChannels {
        &US915_CHANNELS.upstream
    }

    type DownstreamChannels = ChannelPlan;
    fn downstream_channels(&self) -> &Self::DownstreamChannels {
        &US915_CHANNELS.downstream
    }

    fn data_rates(&self) -> &[Modulation] {
        US915_DATARATES
    }

    fn backoff_data_rate(&self, dr_current: DataRate) -> DataRate {
        DataRate(match dr_current.0 {
            0 => 0,
            1 => 0,
            2 => 1,
            3 => 2,
            4 => 3,
            5 => 0,
            6 => 5,
            _ => panic!(),
        })
    }

    fn cflist_type(&self) -> CflistType {
        CflistType::Mask
    }

    fn maximum_payload_size(data_rate: u8) -> Option<u8> {
        Some(match data_rate {
            0 => 19,
            1 => 61,
            2 => 133,
            3 => 230,
            4 => 230,
            5 => 58,
            6 => 133,
            7 => return None,
            8 => 61,
            9 => 137,
            10 => 230,
            11 => 230,
            12 => 230,
            13 => 230,
            _ => return None,
        })
    }

    fn maximum_payload_size_absent_repeaters(data_rate: u8) -> Option<u8> {
        Some(match data_rate {
            0 => 19,
            1 => 61,
            2 => 133,
            3 => 250,
            4 => 250,
            5 => 58,
            6 => 133,
            7 => return None,
            8 => 61,
            9 => 137,
            10 => 250,
            11 => 250,
            12 => 250,
            13 => 250,
            _ => return None,
        })
    }
}

impl Us915 {
    // 2.5.5, mapping from ChMaskCntl to ChMask meaning
    fn link_adr_req_ch_mask_cntl() -> () {
        todo!()
    }

    fn beacon_settings() -> () {
        todo!()
    }
}

const US915_CHANNELS: Channels<Chain<ChannelPlan, ChannelPlan>, ChannelPlan> = Channels {
    upstream: get_move::chain(
        ChannelPlan {
            first_channel: Frequency::from_khz(902_300),
            channel_step: Frequency::from_khz(200),
            count: 64,

            bandwidth: Frequency::from_khz(125),

            data_rate_min: DataRate(0),
            data_rate_max: DataRate(3),
            coding_rate: Some(CodingRate::Cr4_5),
        },
        ChannelPlan {
            first_channel: Frequency::from_khz(903_000),
            channel_step: Frequency::from_khz(1_600),
            count: 8,

            // FIXME: 500kHZ @ Dr4 or 1.5233MHZ @ LR-FHSS
            bandwidth: Frequency::from_khz(500),
            data_rate_max: DataRate(4),
            data_rate_min: DataRate(4),
            coding_rate: None,
        },
    ),

    downstream: ChannelPlan {
        first_channel: Frequency::from_khz(923_300),
        channel_step: Frequency::from_khz(600),
        count: 8,

        bandwidth: Frequency::from_khz(600),

        data_rate_max: DataRate(13),
        data_rate_min: DataRate(8),
        coding_rate: None,
    },
};

const US915_DATARATES: &[Modulation] = &[
    Modulation::Lora {
        sf: 10,
        bw: Frequency::from_khz(125),
    },
    Modulation::Lora {
        sf: 9,
        bw: Frequency::from_khz(125),
    },
    Modulation::Lora {
        sf: 8,
        bw: Frequency::from_khz(125),
    },
    Modulation::Lora {
        sf: 7,
        bw: Frequency::from_khz(125),
    },
    Modulation::Lora {
        sf: 8,
        bw: Frequency::from_khz(500),
    },
    Modulation::LrFhss {
        coding_rate: CodingRate::Cr1_3,
        bandwidth: Frequency::from_khz(1_523),
    },
    Modulation::LrFhss {
        coding_rate: CodingRate::Cr2_3,
        bandwidth: Frequency::from_khz(1_523),
    },
    Modulation::Rfu,
    Modulation::Lora {
        sf: 12,
        bw: Frequency::from_khz(500),
    },
    Modulation::Lora {
        sf: 11,
        bw: Frequency::from_khz(500),
    },
    Modulation::Lora {
        sf: 10,
        bw: Frequency::from_khz(500),
    },
    Modulation::Lora {
        sf: 9,
        bw: Frequency::from_khz(500),
    },
    Modulation::Lora {
        sf: 8,
        bw: Frequency::from_khz(500),
    },
    Modulation::Lora {
        sf: 7,
        bw: Frequency::from_khz(500),
    },
    Modulation::Rfu,
];

/// EU863-870 MHz Band
///
/// As defined by RP002-1.0.3, 2.4 (line 416)
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct Eu868;

impl Band for Eu868 {
    type UpstreamChannels = [ChannelDetails; 3];

    fn upstream_channels(&self) -> &Self::UpstreamChannels {
        &EU863_UPSTREAM_CHANNELS
    }

    type DownstreamChannels = [ChannelDetails; 0];

    fn downstream_channels(&self) -> &Self::DownstreamChannels {
        &EU863_DOWNSTREAM_CHANNELS
    }

    fn data_rates(&self) -> &[Modulation] {
        &EU863_DATARATES[..]
    }

    // Table 9: EU863-870 Data Rate Backoff table
    fn backoff_data_rate(&self, dr_current: DataRate) -> DataRate {
        DataRate(match dr_current.0 {
            0 => 0,
            1 => 1,
            2 => 1,
            3 => 2,
            4 => 3,
            5 => 4,
            6 => 5,
            7 => 6,
            8 => 0,
            9 => 8,
            10 => 0,
            11 => 10,
            // table ends here
            _ => panic!(),
        })
    }

    fn cflist_type(&self) -> CflistType {
        CflistType::Specific
    }

    fn maximum_payload_size(data_rate: u8) -> Option<u8> {
        Some(match data_rate {
            0 => 59,
            1 => 59,
            2 => 59,
            3 => 123,
            4 => 230,
            5 => 230,
            6 => 230,
            7 => 230,
            8 => 58,
            9 => 123,
            10 => 58,
            11 => 123,
            _ => return None,
        })
    }

    fn maximum_payload_size_absent_repeaters(data_rate: u8) -> Option<u8> {
        Some(match data_rate {
            0 => 59,
            1 => 59,
            2 => 59,
            3 => 123,
            4 => 250,
            5 => 250,
            6 => 250,
            7 => 250,
            8 => 58,
            9 => 123,
            10 => 58,
            11 => 123,
            _ => return None,
        })
    }
}

impl Eu868 {
    // Table 14: EU863-870 downlink RX1 data rate mapping
    pub fn rx1_downlink_data_rate(rx1_dr_offset: u8, data_rate: u8) -> Option<u8> {
        todo!()
    }

    // Table 15: EU863-870 beacon settings
    pub fn beacon_settings() -> (CodingRate, DataRate, Polarity) {
        (CodingRate::Cr4_5, DataRate(3), Polarity::Normal)
    }
}

const EU863_DATARATES: [Modulation; 15] = [
    Modulation::Lora {
        sf: 12,
        bw: Frequency::from_khz(125),
    },
    Modulation::Lora {
        sf: 11,
        bw: Frequency::from_khz(125),
    },
    Modulation::Lora {
        sf: 10,
        bw: Frequency::from_khz(125),
    },
    Modulation::Lora {
        sf: 9,
        bw: Frequency::from_khz(125),
    },
    Modulation::Lora {
        sf: 8,
        bw: Frequency::from_khz(125),
    },
    Modulation::Lora {
        sf: 7,
        bw: Frequency::from_khz(125),
    },
    Modulation::Lora {
        sf: 7,
        bw: Frequency::from_khz(250),
    },
    Modulation::Fsk { rate: 50 },
    Modulation::LrFhss {
        coding_rate: CodingRate::Cr1_3,
        bandwidth: Frequency::from_khz(137),
    },
    Modulation::LrFhss {
        coding_rate: CodingRate::Cr2_3,
        bandwidth: Frequency::from_khz(137),
    },
    Modulation::LrFhss {
        coding_rate: CodingRate::Cr1_3,
        bandwidth: Frequency::from_khz(336),
    },
    Modulation::LrFhss {
        coding_rate: CodingRate::Cr2_3,
        bandwidth: Frequency::from_khz(336),
    },
    Modulation::Rfu,
    Modulation::Rfu,
    Modulation::Rfu,
];

const EU863_UPSTREAM_CHANNELS: [ChannelDetails; 3] = [
    ChannelDetails {
        bandwidth: Frequency::from_khz(125),
        frequency: Frequency::from_khz(868_100),
        data_rate_min: DataRate(0),
        data_rate_max: DataRate(5),

        // ???
        coding_rate: None,
    },
    ChannelDetails {
        bandwidth: Frequency::from_khz(125),
        frequency: Frequency::from_khz(868_300),
        data_rate_min: DataRate(0),
        data_rate_max: DataRate(5),

        // ???
        coding_rate: None,
    },
    ChannelDetails {
        bandwidth: Frequency::from_khz(125),
        frequency: Frequency::from_khz(868_500),
        data_rate_min: DataRate(0),
        data_rate_max: DataRate(5),

        // ???
        coding_rate: None,
    },
];

const EU863_DOWNSTREAM_CHANNELS: [ChannelDetails; 0] = [];
