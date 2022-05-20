//! The format of all packets that are part of uplink and downlink, providing encoding and decoding
//!
//! Contains items defined in LoRaWAN L2 1.0.4, 4. MAC Frame Formats and 6. End-Device Activation

use super::DevAddr;
use crate::serde::*;
use aes::Aes128;
use cmac::{Cmac, Mac};
use core::marker::PhantomData;
use modular_bitfield::prelude::*;

// Class A:
// Following each uplink transmission, the end-device SHALL open one or two receive windows
// (RX1 and RX2); if no packet destined for the end-device is received in RX1, it SHALL open
// RX2. The receive windows start times are defined using the end of the transmission as a
// reference, see Figure 2.

///
/// PHYPayload:
/// ```norust
///  1   | 7..M         | 4
/// MHDR | MACPayload   | MIC
///
/// MHDR | Join-Request | MIC
/// MHDR | Join-Accept  | MIC
/// ```
///
/// MACPayload:
/// ```norust
/// FHDR | FPort | FRMPayload
/// ```
///
/// FHDR:
/// ```norust
/// DevAddr | FCtrl | FCnt | FOpts
/// ```
///
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Clone, Copy)]
pub struct PhyPayload<T, S> {
    _type_state: PhantomData<S>,
    bytes: T,
}

pub mod decode_state {
    pub enum Encrypted {}
    pub enum Decrypted {}

    mod sealed {
        pub trait Sealed {}
    }

    pub trait DecodeState: sealed::Sealed {}

    impl sealed::Sealed for Encrypted {}
    impl sealed::Sealed for Decrypted {}
    impl DecodeState for Encrypted {}
    impl DecodeState for Decrypted {}
}

/// ```norust
/// struct PhyPayload {
///     MHDR: u8,
///     MACPayload: [u8;7..],
///     MIC: u32,
/// }
/// ```
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub enum PhyPayloadDecodeError {
    SmallerThanMinSize { have: usize, need: usize },
}

impl<T: AsRef<[u8]>, S> core::fmt::Debug for PhyPayload<T, S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PhyPayload")
            .field("mac_header", &self.mac_header())
            .field("payload_bytes", &self.payload_bytes())
            .field("mic", &self.mic())
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadParseError {
    UnknownMajor { major: u8 },
    JoinRequestParseError(JoinRequestParseError),
    JoinAcceptParseError(JoinAcceptParseError),
}

impl From<JoinRequestParseError> for PayloadParseError {
    fn from(other: JoinRequestParseError) -> Self {
        PayloadParseError::JoinRequestParseError(other)
    }
}

impl From<JoinAcceptParseError> for PayloadParseError {
    fn from(other: JoinAcceptParseError) -> Self {
        PayloadParseError::JoinAcceptParseError(other)
    }
}

// Offsets, etc, that are valid for all decode states
impl<T: AsRef<[u8]>, S> PhyPayload<T, S> {
    fn bytes(&self) -> &[u8] {
        self.bytes.as_ref()
    }

    pub fn mac_header(&self) -> MacHeader {
        MacHeader::from_bytes(self.bytes()[0..1].try_into().unwrap())
    }

    pub fn payload_bytes(&self) -> &[u8] {
        let end = self.bytes().len() - 4;
        &self.bytes()[1..end]
    }

    pub fn mic(&self) -> [u8; 4] {
        let start = self.bytes().len() - 4;
        self.bytes()[start..].try_into().unwrap()
    }
}

impl<T: AsRef<[u8]>> PhyPayload<T, decode_state::Encrypted> {
    pub fn payload(&self) -> Result<Payload<'_>, PayloadParseError> {
        let mh = self.mac_header();
        if mh.major() != 0 {
            return Err(PayloadParseError::UnknownMajor { major: mh.major() });
        }

        let bytes = self.payload_bytes();

        Ok(match mh.ftype() {
            FrameType::JoinRequest => Payload::JoinRequest(JoinRequest::from_bytes(bytes)?),
            FrameType::JoinAccept => Payload::JoinAccept(JoinAccept::from_bytes(bytes)?),
            _ => Payload::MacPayload(MacPayload { bytes }),
        })
    }

    /*
    JoinNonce is a non-repeating value provided by the Join Server and used by the end-device
    to derive the two session keys NwkSKey and AppSKey, which SHALL be calculated as
    follows:10

    NwkSKey = aes128_encrypt(AppKey, 0x01 | JoinNonce | NetID | DevNonce | pad16)
    AppSKey = aes128_encrypt(AppKey, 0x02 | JoinNonce | NetID | DevNonce | pad16)
    The MIC value for a Join-Accept frame SHALL be calculated as follows:11

    CMAC = aes128_cmac(AppKey, MHDR | JoinNonce | NetID | DevAddr |
    DLSettings | RXDelay | CFList)
    MIC = CMAC[0..3]
    The Join-Accept frame itself SHALL be encrypted with the AppKey as follows:

    aes128_decrypt(AppKey, JoinNonce | NetID | DevAddr | DLSettings | RXDelay |
    CFList | MIC)
    */
    pub fn mic_expected(&self, app_key: &[u8]) -> [u8; 4] {
        let mhdr = self.mac_header();
        match mhdr.ftype() {
            // NOTE: this is not required for end-devices
            FrameType::JoinRequest => {
                // for Join-Request:
                //   CMAC = aes128_cmac(AppKey, MHDR | JoinEUI | DevEUI | DevNonce)
                //   MIC = CMAC[0..3]
                //
                let mut mac = Cmac::<Aes128>::new_from_slice(app_key).unwrap();
                let end = self.bytes().len() - 4;
                mac.update(&self.bytes()[..end]);
                mac.finalize().into_bytes().as_slice()[..4]
                    .try_into()
                    .unwrap()
            }
            FrameType::JoinAccept => {
                // CMAC = aes128_cmac(AppKey, MHDR | JoinNonce | NetID | DevAddr | DLSettings | RXDelay | CFList)
                // MIC = CMAC[0..3]
                //
                // FIXME: this needs a decrypted frame!
                let mut mac = Cmac::<Aes128>::new_from_slice(app_key).unwrap();
                let end = self.bytes().len() - 4;
                mac.update(&self.bytes()[..end]);
                mac.finalize().into_bytes().as_slice()[..4]
                    .try_into()
                    .unwrap()
            }
            _ => todo!(),
        }
    }

    pub fn from_bytes(bytes: T) -> Result<Self, PhyPayloadDecodeError> {
        let b = bytes.as_ref();
        let have = b.len();
        {
            let need = 1 + 7 + 4;
            if have < need {
                return Err(PhyPayloadDecodeError::SmallerThanMinSize { have, need });
            }
        }

        Ok(Self {
            _type_state: PhantomData,
            bytes,
        })
    }
}

/// MHDR
#[bitfield]
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacHeader {
    #[bits = 3]
    pub ftype: FrameType,
    pub rfu: B3,
    pub major: B2,
}

/// FType, 3 bits
#[repr(u8)]
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, BitfieldSpecifier)]
pub enum FrameType {
    /// Join-Request
    JoinRequest = 0b000,
    /// Join-Accept
    JoinAccept = 0b001,
    UnconfirmedDataUplink = 0b010,
    UnconfirmedDataDownlink = 0b011,
    ConfirmedDataUplink = 0b100,
    ConfirmedDataDownlink = 0b101,
    Rfu = 0b110,
    Proprietary = 0b111,
}

#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrameHeaderParseError {
    TooSmall { have: usize },
}

#[derive(Clone, Copy)]
pub struct FrameHeader<'a> {
    pub bytes: &'a [u8],
}

impl<'a> FrameHeader<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, FrameHeaderParseError> {
        let have = bytes.len();
        let need = 4 + 1 + 1;
        if have < need {
            return Err(FrameHeaderParseError::TooSmall { have });
        }

        Ok(Self { bytes })
    }
}

/// FHDR
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct FrameHeaderBuf {
    /// DevAddr
    pub dev_addr: [u8; 4],

    /// FCtrl
    pub fctrl: u8,
    /// FCnt
    pub frame_count: u8,

    // FIXME: 0 to 15 bytes
    /// FOpts
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
    /// `FOptsLen`
    pub frame_opts_len: B4,
}

#[bitfield]
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct UplinkFrameControl {
    pub adr: bool,
    pub adr_ack_req: bool,
    pub ack: bool,

    /// Set true by the end-device to indicate to the Network Server that the end-device has
    /// enabled class B and is now ready to receive scheduled downlink pings.
    pub class_b: bool,

    /// `FOptsLen`: actual length of the frame options field (`FOpts`) included in the frame.
    pub frame_opts_len: B4,
}

#[derive(Clone, Copy)]
pub enum Payload<'a> {
    MacPayload(MacPayload<'a>),
    JoinRequest(JoinRequest<'a>),
    // FIXME: JoinAccept needs to be decrypted!
    JoinAccept(JoinAccept<'a>),
}

/// MACPayload
///
/// ```norust
/// struct MACPayload {
///    FHDR: [u8; Q], where Q: 7..=22,
///    FPort: Option<u8>,
///    FRMPayload: [u8; P], where P: 0..N
/// }
/// ```
///
/// # Encryption
///
/// `frame_payload` is encrypted when sent/recvd. Encryption is AES-CTR-like, with the following
/// used as the counter:
///
/// ```norust
/// ---
/// size (octets) -> 1    |  4       | 1     | 4         | 4      | 1    | 1
/// A_i           -> 0x01 | 4 * 0x00 | `Dir` | `DevAddr` | `FCnt` | 0x00 | i
/// ---
///
/// S_i = aes128_ecb(K, A_i) for i 1..k
///   where `K = frame_port_key(FPort)`
///         `k = ceil(plaintext_payload.len() / 16)`
/// S = S_1 | S_2 | ... | S_k
///
/// frame_payload_pad = (plaintext_payload | pad_16) ^ S
/// frame_payload = frame_payload_pad[0..plaintext_payload.len()]
/// ```
///
/// # Message Integrity Code
///
/// NOTE: FRMPayload here is encrypted
///
/// ```norust
/// msg = MHDR | FHDR | FPort | FRMPayload
/// CMAC = aes128_cmac(NwKSKey, B_0 | msg)
/// MIC = CMAC[0..=3]
/// B_0 = 0x49 | 4* 0x00 | Dir | DevAddr | FCnt | 0x00 | msg.len()
/// ```
///
///
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct MacPayload<'a> {
    pub bytes: &'a [u8],
}

impl<'a> MacPayload<'a> {
    pub fn fhdr_bytes(&self) -> &'a [u8] {
        // examine fhdr and determine length
        todo!()
    }

    pub fn fport(&self) -> Option<u8> {
        todo!()
    }

    pub fn frm_paylod_bytes(&self) -> &'a [u8] {
        todo!()
    }
}

#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub enum Key {
    NwkSKey,
    AppSKey,
}

pub fn frame_port_key(frame_port: u8) -> Key {
    if frame_port == 0 {
        Key::NwkSKey
    } else {
        Key::AppSKey
    }
}

#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct JoinRequestBuf {
    pub join_eui: u64,
    pub dev_eui: u64,
    /// Set to 0 when the end-device is powered up and incrimented with every Join-Request
    /// A given `DevNonce` shall never be reused for a given JoinEUI value. If the end-device can
    /// be power cycled, then `DevNonce` shall be persistent.
    pub dev_nonce: u16,
}

#[derive(Clone, Copy)]
pub struct JoinRequest<'a> {
    pub bytes: &'a [u8],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinRequestParseError {
    SizeMismatch { have: usize, need: usize },
}

impl<'a> JoinRequest<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, JoinRequestParseError> {
        let need = 8 + 8 + 2;
        let have = bytes.len();

        if have != need {
            return Err(JoinRequestParseError::SizeMismatch { have, need });
        }

        Ok(Self { bytes })
    }

    pub fn join_eui(&self) -> u64 {
        u64::from_le_bytes(self.bytes[0..8].try_into().unwrap())
    }

    pub fn dev_eui(&self) -> u64 {
        u64::from_le_bytes(self.bytes[8..16].try_into().unwrap())
    }

    pub fn dev_nonce(&self) -> u16 {
        u16::from_le_bytes(self.bytes[16..18].try_into().unwrap())
    }

    pub fn to_owned(&self) -> JoinRequestBuf {
        JoinRequestBuf {
            dev_eui: self.dev_eui(),
            dev_nonce: self.dev_nonce(),
            join_eui: self.join_eui(),
        }
    }
}

impl<'a> core::fmt::Debug for JoinRequest<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("JoinRequest")
            .field("join_eui", &self.join_eui())
            .field("dev_eui", &self.dev_eui())
            .field("dev_nonce", &self.dev_nonce())
            .finish()
    }
}

#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct JoinAcceptBuf {
    pub join_nonce: [u8; 3],
    pub net_id: [u8; 3],
    pub dev_addr: DevAddr,
    pub dl_settings: DlSettings,
    pub rx_delay: u8,

    pub cf_list: Option<()>,
}

#[derive(Clone, Copy)]
pub struct JoinAccept<'a> {
    pub bytes: &'a [u8],
}

#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinAcceptParseError {
    SizeMismatch { have: usize },
}

impl<'a> JoinAccept<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, JoinAcceptParseError> {
        let need1 = 3 + 3 + 4 + 1 + 1;
        let need2 = need1 + 16;
        let have = bytes.len();

        if need1 != have && need2 != have {
            return Err(JoinAcceptParseError::SizeMismatch { have });
        }

        Ok(Self { bytes })
    }

    pub fn join_nonce(&self) -> u32 {
        u24_from_le_bytes(self.bytes[0..3].try_into().unwrap())
    }

    pub fn net_id(&self) -> u32 {
        u24_from_le_bytes(self.bytes[3..6].try_into().unwrap())
    }

    pub fn dev_addr(&self) -> u32 {
        u32::from_le_bytes(self.bytes[6..10].try_into().unwrap())
    }

    pub fn dl_settings(&self) -> DlSettings {
        DlSettings::from_bytes(self.bytes[10..=10].try_into().unwrap())
    }

    pub fn rx_delay(&self) -> u8 {
        self.bytes[11]
    }

    pub fn cf_list(&self) -> Option<&[u8]> {
        let need = 3 + 3 + 4 + 1 + 1;
        if self.bytes.len() != need {
            None
        } else {
            Some(&self.bytes[need..])
        }
    }

    /// NwkSKey
    pub fn calculate_network_session_key(&self) -> [u8; 16] {
        todo!()
    }

    /// AppSKey
    pub fn calculate_app_session_key(&self) -> [u8; 16] {
        todo!()
    }
}

#[bitfield]
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct DlSettings {
    pub rfu: bool,
    pub rx1_dr_offset: B3,
    pub rx2_data_rate: B4,
}
