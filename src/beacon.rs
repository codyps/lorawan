use crate::Sf;
use crc_0x8810::CRC_16_LORA;
use modular_bitfield::prelude::*;

// crc notes:
//
// P(x) = x16 + x12 + x5 + x0
// explcit + 1 form:
// poly = 0b0001_0000_0010_0001
//          5432 1098 7654 3210
//          1111 11
//
// normal form:
// poly = 0b0000_1000_0001_0000
//      =
//
//
// x^10 +x^7 +x^3 +x^2 +x +1
// 0b0000_0010_0100_0111
//   5432 1098 7654 3210
//   1111 11

// TODO: consider if we should pack crcs and/or RFU fields into Beacon
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Beacon {
    pub param: Param,
    pub time: u32,
    pub gw_specific: GwSpecificKind,
}

#[bitfield]
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Param {
    pub rfu: B6,
    pub prec: B2,
}

#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    WrongSize { need: usize, actual: usize },
    Crc1Wrong,
    Crc2Wrong,
}

impl Beacon {
    pub fn parse_beacon(sf: Sf, bytes: &[u8]) -> Result<Self, Error> {
        let need = sf.beacon_bytes();
        let actual = bytes.len();
        if actual != need {
            return Err(Error::WrongSize { need, actual });
        }

        let rem = bytes;
        let rfu1_bytes_ct = sf.beacon_rfu1_bytes();
        let _rfu1_bytes = &rem[0..rfu1_bytes_ct];
        let rem = &rem[rfu1_bytes_ct..];
        let param = rem[0];
        let rem = &rem[1..];
        let time = u32::from_le_bytes(rem[0..4].try_into().unwrap());
        let rem = &rem[4..];
        let crc1 = u16::from_le_bytes(rem[0..2].try_into().unwrap());
        let rem = &rem[2..];
        let crc1_end = rfu1_bytes_ct + 1 + 4;
        let crc1_calc = CRC_16_LORA.checksum(&bytes[0..crc1_end]);

        if crc1 != crc1_calc {
            return Err(Error::Crc1Wrong);
        }

        let gw_specific = GwSpecificKind::from_bytes(rem[0..7].try_into().unwrap());
        let rem = &rem[7..];
        let rfu2_bytes_ct = sf.beacon_rfu2_bytes();
        let _rfu2_bytes = &rem[0..rfu2_bytes_ct];
        let rem = &rem[rfu2_bytes_ct..];
        let crc2 = u16::from_le_bytes(rem.try_into().unwrap());
        let crc2_start = crc1_end + 2;
        let crc2_end = crc2_start + 7 + rfu2_bytes_ct;
        let crc2_calc = CRC_16_LORA.checksum(&bytes[crc2_start..crc2_end]);

        if crc2 != crc2_calc {
            return Err(Error::Crc2Wrong);
        }

        Ok(Beacon {
            param: Param { bytes: [param] },
            time,
            gw_specific,
        })
    }
}

#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GwSpecific {
    bytes: [u8; 7],
}

#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GwSpecificKind {
    Gps {
        antenna: Antenna,
        lat: u32,
        long: u32,
    },
    NetIdAndGatewayId {
        net_id: u32,
        gateway_id: u32,
    },
    /// 4..=127
    Rfu {
        info_desc: u8,
        info: [u8; 6],
    },
    /// 128..=255
    NetworkSpecific {
        info_desc: u8,
        info: [u8; 6],
    },
}

fn u24_from_le_bytes(bytes: [u8; 3]) -> u32 {
    ((bytes[2] as u32) << 16) | ((bytes[1] as u32) << 8) | (bytes[0] as u32)
}

fn u24_to_le_bytes(v: u32) -> [u8; 3] {
    assert!((v & 0xFF_00_00_00) == 0);

    [
        (v & 0xff) as u8,
        ((v & 0xff_00) >> 8) as u8,
        ((v & 0xff_00_00) >> 16) as u8,
    ]
}

impl GwSpecificKind {
    pub fn as_bytes(&self) -> [u8; 7] {
        todo!()
    }

    pub fn from_bytes(bytes: [u8; 7]) -> Self {
        let n = InfoDesc::from_byte(bytes[0]);

        match n {
            InfoDesc::GpsCoordinate { antenna } => {
                let lat = u24_from_le_bytes(bytes[1..4].try_into().unwrap());
                let long = u24_from_le_bytes(bytes[4..7].try_into().unwrap());
                Self::Gps { antenna, lat, long }
            }
            InfoDesc::NetworkSpecific { info_desc } => Self::NetworkSpecific {
                info_desc,
                info: bytes[1..].try_into().unwrap(),
            },
            InfoDesc::NetIdAndGatewayId => {
                let net_id = u24_from_le_bytes(bytes[1..4].try_into().unwrap());
                let gateway_id = u24_from_le_bytes(bytes[4..7].try_into().unwrap());
                Self::NetIdAndGatewayId { net_id, gateway_id }
            }
            InfoDesc::Rfu { info_desc } => Self::Rfu {
                info_desc,
                info: bytes[1..].try_into().unwrap(),
            },
        }
    }
}

#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Antenna {
    First,
    Second,
    Third,
}

#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfoDesc {
    GpsCoordinate { antenna: Antenna },
    NetIdAndGatewayId,
    Rfu { info_desc: u8 },
    NetworkSpecific { info_desc: u8 },
}

impl InfoDesc {
    pub fn from_byte(info_desc: u8) -> Self {
        match info_desc {
            0 => Self::GpsCoordinate {
                antenna: Antenna::First,
            },
            1 => Self::GpsCoordinate {
                antenna: Antenna::Second,
            },
            2 => Self::GpsCoordinate {
                antenna: Antenna::Third,
            },
            3 => Self::NetIdAndGatewayId,
            4..=127 => Self::Rfu { info_desc },
            128..=255 => Self::NetworkSpecific { info_desc },
        }
    }
}
