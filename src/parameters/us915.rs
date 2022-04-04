use super::*;

/// US902-928 MHz ISM Band
///
/// As defined by RP002-1.0.3, 2.5 (line 531)
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct Us915;

impl Band for Us915 {
    fn channel_mask_apply(
        &self,
        channel_mask_cntl: u8,
        channel_mask: u16,
        current_channel_mask: u128,
    ) -> Result<u128, ()> {
        match channel_mask_cntl {
            0..=4 => {
                let shift = channel_mask_cntl * 16;
                let mask = current_channel_mask & (u16::MAX as u128) << shift;
                let mask = mask | (channel_mask as u128) << shift;
                Ok(mask)
            }
            5 => {
                // TODO: there's probably a nice way to simplify this
                let mut mask = current_channel_mask;
                for i in 0..7 {
                    let shift = i * 16;
                    let m = (u16::MAX as u128) << shift;
                    if i & channel_mask != 0 {
                        mask |= m;
                    } else {
                        mask &= m;
                    }
                }

                Ok(mask)
            }
            6 | 7 => {
                // the first 64 channels are 128kHz in the standard channel plan
                let mask_125khz = if channel_mask_cntl == 6 {
                    u64::MAX as u128
                } else {
                    0
                };

                Ok(mask_125khz | ((channel_mask as u128) << 64))
            }
            8..=255 => Err(()),
        }
    }

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

    fn backoff_data_rate(&self, dr_current: DataRate) -> Option<DataRate> {
        Some(
            match dr_current.into() {
                0 => 0,
                1 => 0,
                2 => 1,
                3 => 2,
                4 => 3,
                5 => 0,
                6 => 5,
                7..=255 => return None,
            }
            .try_into()
            .unwrap(),
        )
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
            14..=255 => return None,
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
            14..=255 => return None,
        })
    }

    fn beacon_settings(&self) -> &BeaconSettings {
        const BEACON_SETTINGS: BeaconSettings = BeaconSettings {
            dr: DataRate::_4,
            cr: CodingRate::Cr4_5,
            polarity: Polarity::Normal,
            channels: ChannelSpec::AllDownstream,
        };
        &BEACON_SETTINGS
    }

    fn rx1_recv_channel(transmit_channel: u8) -> u8 {
        transmit_channel % 8
    }

    fn rx1_window_data_rate(upstream_datarate: DataRate, rx1_dr_offset: u8) -> Option<DataRate> {
        const DR_MAP: [[u8; 4]; 7] = [
            [10, 9, 8, 8],
            [11, 10, 9, 8],
            [12, 11, 10, 9],
            [13, 12, 11, 10],
            [13, 13, 12, 11],
            [10, 9, 8, 8],
            [11, 10, 9, 8],
        ];

        DR_MAP
            .get(Into::<u8>::into(upstream_datarate) as usize)
            .and_then(|v| v.get(rx1_dr_offset as usize))
            .copied()
            .map(|x| x.try_into().unwrap())
    }

    fn rx2_window_details(&self) -> (Frequency, DataRate) {
        (Frequency::from_khz(923_300), DataRate::_0)
    }
}

/*
impl Us915 {
    // 2.5.5, mapping from ChMaskCntl to ChMask meaning
    fn link_adr_req_ch_mask_cntl() -> () {
        todo!()
    }
}
*/

const US915_CHANNELS: Channels<Chain<ChannelPlan, ChannelPlan>, ChannelPlan> = Channels {
    upstream: get_move::chain(
        ChannelPlan {
            first_channel: Frequency::from_khz(902_300),
            channel_step: Frequency::from_khz(200),
            count: 64,

            bandwidth: Frequency::from_khz(125),

            data_rate_min: DataRate::_0,
            data_rate_max: DataRate::_3,
            coding_rate: Some(CodingRate::Cr4_5),
        },
        ChannelPlan {
            first_channel: Frequency::from_khz(903_000),
            channel_step: Frequency::from_khz(1_600),
            count: 8,

            // FIXME: 500kHZ @ Dr4 or 1.5233MHZ @ LR-FHSS
            bandwidth: Frequency::from_khz(500),
            data_rate_max: DataRate::_4,
            data_rate_min: DataRate::_4,
            coding_rate: None,
        },
    ),

    downstream: ChannelPlan {
        first_channel: Frequency::from_khz(923_300),
        channel_step: Frequency::from_khz(600),
        count: 8,

        bandwidth: Frequency::from_khz(600),

        data_rate_max: DataRate::_13,
        data_rate_min: DataRate::_8,
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
