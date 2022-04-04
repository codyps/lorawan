use super::DevAddr;
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
#[derive(Debug, Clone, Copy)]
pub struct PhyPayload {}

/// FHDR
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct Fhdr {}

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

/// FHDR
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct FrameHeader {
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

/// MACPayload
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
    // FHDR
    /// FPort, 0..1
    pub frame_port: Option<u8>,
    /// FRMPayload, 0..N
    pub frame_payload: &'a [u8],
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
pub struct JoinRequest {
    pub join_eui: u64,
    pub dev_eui: u64,
    /// Set to 0 when the end-device is powered up and incrimented with every Join-Request
    /// A given `DevNonce` shall never be reused for a given JoinEUI value. If the end-device can
    /// be power cycled, then `DevNonce` shall be persistent.
    pub dev_nonce: u16,
}

#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct JoinAccept {
    pub join_nonce: [u8; 3],
    pub net_id: [u8; 3],
    pub dev_addr: DevAddr,
    pub dl_settings: DlSettings,
    pub rx_delay: u8,

    pub cf_list: Option<()>,
}

#[bitfield]
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct DlSettings {
    pub rfu: bool,
    pub rx1_dr_offset: B3,
    pub rx2_data_rate: B4,
}
