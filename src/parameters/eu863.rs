use super::*;

/// EU863-870 MHz Band
///
/// As defined by RP002-1.0.3, 2.4 (line 416)
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct Eu868;

impl Band for Eu868 {
    fn channel_mask_apply(
        &self,
        channel_mask_cntl: u8,
        channel_mask: u16,
        current_channel_mask: u128,
    ) -> Result<u128, ()> {
        match channel_mask_cntl {
            0 => {
                let mask = current_channel_mask & u16::MAX as u128;
                let mask = mask | channel_mask as u128;
                Ok(mask)
            }
            // RFU
            1 | 2 | 3 | 4 | 5 => Err(()),
            6 => Ok(u128::MAX),
            7..=255 => Err(()),
        }
    }

    type UpstreamChannels = [ChannelDetails; 3];

    // XXX: consider if there's a better representation for upstream/downstream for Bands that have
    // identical upstream and downstream channels
    fn upstream_channels(&self) -> &Self::UpstreamChannels {
        &EU863_CHANNELS
    }

    type DownstreamChannels = [ChannelDetails; 3];

    fn downstream_channels(&self) -> &Self::DownstreamChannels {
        &EU863_CHANNELS
    }

    fn data_rates(&self) -> &[Modulation] {
        &EU863_DATARATES[..]
    }

    // Table 9: EU863-870 Data Rate Backoff table
    fn backoff_data_rate(&self, dr_current: DataRate) -> Option<DataRate> {
        Some(
            match dr_current.into() {
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
                _ => return None,
            }
            .try_into()
            .unwrap(),
        )
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

    // Table 15: EU863-870 beacon settings
    fn beacon_settings(&self) -> &BeaconSettings {
        const BEACON_SETTINGS: BeaconSettings = BeaconSettings {
            cr: CodingRate::Cr4_5,
            dr: DataRate::_3,
            polarity: Polarity::Normal,
            channels: ChannelSpec::One(Frequency::from_khz(434_665)),
        };
        &BEACON_SETTINGS
    }

    /// Table 14: EU863-870 downlink RX1 data rate mapping
    fn rx1_window_data_rate(upstream_datarate: DataRate, rx1_dr_offset: u8) -> Option<DataRate> {
        const DR_MAP: [[u8; 6]; 12] = [
            [0, 0, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0],
            [2, 1, 0, 0, 0, 0],
            [3, 2, 1, 0, 0, 0],
            [4, 3, 2, 1, 0, 0],
            [5, 4, 3, 2, 1, 0],
            [6, 5, 4, 3, 2, 1],
            [7, 6, 5, 4, 3, 2],
            [1, 0, 0, 0, 0, 0],
            [2, 1, 0, 0, 0, 0],
            [1, 0, 0, 0, 0, 0],
            [2, 1, 0, 0, 0, 0],
        ];

        DR_MAP
            .get(Into::<u8>::into(upstream_datarate) as usize)
            .and_then(|v| v.get(rx1_dr_offset as usize))
            .copied()
            .map(|x| x.try_into().unwrap())
    }

    /// "By default, the RX1 receive window uses the same channel as the preceding uplink"
    fn rx1_recv_channel(transmit_channel: u8) -> u8 {
        transmit_channel
    }

    // rx2 fixed frequency and datarate
    // 869.525/DR0
    fn rx2_window_details(&self) -> (Frequency, DataRate) {
        (Frequency::from_khz(869_525), DataRate::_0)
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

const EU863_CHANNELS: [ChannelDetails; 3] = [
    ChannelDetails {
        bandwidth: Frequency::from_khz(125),
        frequency: Frequency::from_khz(868_100),
        data_rate_min: DataRate::_0,
        data_rate_max: DataRate::_5,

        // ???
        coding_rate: None,
    },
    ChannelDetails {
        bandwidth: Frequency::from_khz(125),
        frequency: Frequency::from_khz(868_300),
        data_rate_min: DataRate::_0,
        data_rate_max: DataRate::_5,

        // ???
        coding_rate: None,
    },
    ChannelDetails {
        bandwidth: Frequency::from_khz(125),
        frequency: Frequency::from_khz(868_500),
        data_rate_min: DataRate::_0,
        data_rate_max: DataRate::_5,

        // ???
        coding_rate: None,
    },
];
