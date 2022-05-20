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

// XXX: consider using the embedded_time Duration instead of core::time
use core::{marker::PhantomData, time::Duration};
use embedded_time::{Clock, Instant};

pub use parameters::*;

pub mod state;

pub mod mac;
pub mod mac_frame;
mod serde;

pub mod beacon;
pub use beacon::Beacon;

// epoch of time is Jan 6, 1980 (GPS)

#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub enum Sf {
    _8,
    _9,
    _10,
    _11,
    _12,
}

const BEACON_BASE_SIZE: usize = 1 + 4 + 2 + 7 + 2;

impl Sf {
    pub const fn beacon_rfu1_bytes(&self) -> usize {
        match *self {
            Sf::_8 => 0,
            Sf::_9 => 1,
            Sf::_10 => 2,
            Sf::_11 => 3,
            Sf::_12 => 4,
        }
    }

    pub const fn beacon_rfu2_bytes(&self) -> usize {
        match *self {
            Sf::_8 => 3,
            Sf::_9 => 0,
            Sf::_10 => 1,
            Sf::_11 => 2,
            Sf::_12 => 3,
        }
    }

    /// 13.2 Beacon Frame Format
    pub const fn beacon_bytes(&self) -> usize {
        BEACON_BASE_SIZE + self.beacon_rfu1_bytes() + self.beacon_rfu2_bytes()
    }
}

/// An index into a per-band table of modulation modes
///
/// This index is returned by various functions instead of specifying the modulation directly. Only
/// values 0 to 15 are valid.
#[allow(clippy::just_underscores_and_digits)]
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
pub struct DataRateOutOfRange;

impl TryFrom<u8> for DataRate {
    type Error = DataRateOutOfRange;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use DataRate::*;
        Ok(match value {
            0 => _0,
            1 => _1,
            2 => _2,
            3 => _3,
            4 => _4,
            5 => _5,
            6 => _6,
            7 => _7,
            8 => _8,
            9 => _9,
            10 => _10,
            11 => _11,
            12 => _12,
            13 => _13,
            14 => _14,
            15 => _15,
            _ => return Err(DataRateOutOfRange),
        })
    }
}

impl From<DataRate> for u8 {
    // XXX: this allow probably shouldn't be required here, but it seems like clippy is
    // misinterpreting these enum variants as plain match patterns, causing it to think we're
    // declaring a bunch of weird names.
    // NOTE: also, placing this just on the `match` block doesn't work. Seems to be another clippy
    // issue/limitation
    #[allow(clippy::just_underscores_and_digits)]
    fn from(v: DataRate) -> Self {
        use DataRate::*;
        match v {
            _0 => 0,
            _1 => 1,
            _2 => 2,
            _3 => 3,
            _4 => 4,
            _5 => 5,
            _6 => 6,
            _7 => 7,
            _8 => 8,
            _9 => 9,
            _10 => 10,
            _11 => 11,
            _12 => 12,
            _13 => 13,
            _14 => 14,
            _15 => 15,
        }
    }
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
pub fn data_rate_backoff(params: &Parameters, adr_ack_cnt: u8) -> BackoffDetails {
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

///
/// `Clock` must be a monotonic clock of with variance of XXX and accuracy of XXX
///
/// What does an EndDevice need to allow clients to do?
///
///  - ClassB: allow changing the periodicity of ping slots (at any time)
///  - ClassB: notify about synchronization with the Network being lost, allow the user to
///    determine the responce (for example, re-enabling ClassB
///
/// What does an EndDevice need to do automatically?
///
///  - ClassB: handle `PingSlotChannelReq` and automatically reply with `PingSlotChannelAns`
///  - ClassB: schedule radio reception durring ping-slot times. Must account for clock drift
///  -
///
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone)]
pub struct EndDevice<C: Clock> {
    /// The current band in use
    // NOTE: using the enum allows us to avoid having either dyn pointers & box or having to make
    // ggthis generic over regions (preventing region transitions unless box/dyn is used)
    pub band_id: Option<parameters::BandId>,

    /// FCntUp: Incremented by an end-device when a data frame is transmitted to a Network Server (uplink).
    ///
    /// - Over the air activated devices (OOTA): set to 0 when a JoinAccept is succesfully processed
    /// - Activation by personalization (ABP): set to 0 by manufacturer. Never otherwise reset.
    ///   Must be persisted for lifetime of device.
    pub frame_count_uplink: u32,

    /// XXX
    pub adr_ack_cnt: u32,

    /// ClassB only
    pub class_b_resp_timeout: Duration,

    /// delays since a previous uplink where the EndDevice will have it's reciever enabled and be
    /// able to recieve downlink from the Network Server.
    ///
    /// NOTE: it happens that the default for all regions is currently 1s. This is only a
    /// recomendation though, and we should expect that at some point this will become a
    /// band/region parameter.
    pub receive_delay1: Duration,

    /// transmit time in the past from which `receive_delay1` and `receive_delay2` are counted.
    /// Determines when we're going to open the class A recv windows (based on those 2 delays).
    pub previous_transmit_time: Option<Instant<C>>,

    pub num_transmits: u8,

    ///
    pub maximum_tx_power: Option<u8>,

    /// bitmask indicating if a uplink channel may be used (bit is set if channel is usable). By
    /// default, all channels are considered usable.
    ///
    /// NOTE: specified regions/bands only appear to have up to 72 channels, allowing a 128 bit value
    /// to represent a bitmask of all possible channels. It might be possible to shrink this
    /// slightly be using `(u64, u16)` or similar if desirable.
    pub uplink_channel_mask: u128,

    /// Set by the `DutyCycleReq` mac command from the Network Server.
    ///
    /// FIXME: needs units.
    pub max_duty_cycle: u8,
}

impl<C: Clock> Default for EndDevice<C> {
    fn default() -> Self {
        Self {
            band_id: None,
            frame_count_uplink: 0,
            adr_ack_cnt: 0,
            // FIXME: not sure this is the right default,
            class_b_resp_timeout: Duration::from_secs(1),

            receive_delay1: Duration::from_secs(1),
            previous_transmit_time: None,

            // FIXME: arbitrary. this should be from the spec instead
            num_transmits: 0,

            maximum_tx_power: None,
            uplink_channel_mask: u128::MAX,
            max_duty_cycle: 0,
        }
    }
}

impl<C> EndDevice<C>
where
    C: Clock,
{
    // NOTE: this function exists to allow us to change receive_delay1 to be a band/region
    // defaulted parameter (which it technically is in the specification). All regions at the
    // moment define it to be the same value (1s) though.
    pub fn receive_delay1(&self) -> Duration {
        self.receive_delay1
    }

    pub fn receive_delay2(&self) -> Duration {
        Duration::from_secs(1) + self.receive_delay1()
    }

    pub fn band(&self) -> Option<impl parameters::Band> {
        Some(match self.band_id {
            Some(parameters::BandId::US915) => parameters::Us915,
            _ => return None,
        })
    }

    /// Change the band_id for this instance
    ///
    /// This might be done in responce to user input that the band to operate in has changed, or if
    /// the location of the device has changed (or been discovered) to lie within a region with a
    /// particular band (location to band mapping is not currently included in this library)
    pub fn set_band_id(&mut self, band_id: Option<BandId>) {
        // TODO: this _probably_ means we need to reset some internal state machines (as we're
        // _probably_ no longer Joined) and as a result a need to re-schedule uplinks and downlinks
        // & reconfigure the radio hardware to support that.
        self.band_id = band_id;
    }

    pub fn send_uplink_unconfirmed(&mut self, payload: &[u8]) -> Result<(), ()> {
        // TODO: schedule a transmition as soon as permitted
        // TODO: construct packet around payload
        todo!()
    }

    pub fn send_uplink_confirmed(&mut self, payload: &[u8]) -> Result<(), ()> {
        todo!()
    }

    pub fn process_mac_request(
        &mut self,
        // TODO: consider having `message_recv_meta` and `mac_message` be the contained in the same structure
        message_recv_meta: MessageRecvMeta<C>,
        mac_message: mac::ReqFromNetworkServer,
    ) -> Result<(), ()> {
        match mac_message {
            mac::ReqFromNetworkServer::LinkAdr(link_adr) => {
                if link_adr.tx_power() != 0xf {
                    // FIXME: map this to some meaningful units
                    self.maximum_tx_power = Some(link_adr.tx_power());
                }

                let channel_mask_ack = if let Some(band) = self.band() {
                    // TODO: consider what should be done if we lack a band/region

                    // TODO: must check if all channels would be disabled
                    // TODO: must check if the channel mask is incompatible with the resulting data
                    // rate or TX power
                    // TODO: must check if the channel mask enables a yet undefined channel
                    if let Ok(uplink_channel_mask) = band.channel_mask_apply(
                        link_adr.channel_mask_ctrl(),
                        link_adr.ch_mask(),
                        self.uplink_channel_mask,
                    ) {
                        self.uplink_channel_mask = uplink_channel_mask;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

                // TODO: link_adr.data_rate
                // TODO: link_adr.redudancy

                self.send_mac_answer(mac::AnsFromEndDevice::LinkAdr(
                    mac::LinkAdrAns::new()
                        .with_channel_mask_ack(channel_mask_ack)
                        .with_power_ack(true)
                        .with_data_rate_ack(true),
                ))
            }
            mac::ReqFromNetworkServer::DevStatus => {
                todo!();
            }
            mac::ReqFromNetworkServer::DutyCycle(duty_cycle) => {
                self.max_duty_cycle = duty_cycle.max_duty_cycle();

                self.send_mac_answer(mac::AnsFromEndDevice::DutyCycle)
            }
            mac::ReqFromNetworkServer::DlChannel(dl_channel) => {
                todo!();
            }
            mac::ReqFromNetworkServer::NewChannel(new_channel_req) => {
                todo!();
            }
            mac::ReqFromNetworkServer::RxParamSetup(rx_param_setup_req) => {
                todo!();
            }
            mac::ReqFromNetworkServer::TxParamSetup(tx_param_setup_req) => {
                todo!();
            }
            mac::ReqFromNetworkServer::RxTimingSetup(rx_timing_setup_req) => {
                todo!();
            }
        }
    }

    pub fn send_mac_answer(&mut self, mac_answer: mac::AnsFromEndDevice) -> Result<(), ()> {
        todo!()
    }
}

/// Meta radio reciever provides about a recieved message
#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone)]
pub struct MessageRecvMeta<C: Clock> {
    pub power_db: u32,
    pub time: Instant<C>,
    // TODO: consider if in some cases we need to record modulation information here
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

/// Calculate the NwkSKey on the end-device
pub fn end_device_network_skey(join_nonce: [u8; 3], net_id: [u8; 3], dev_nonce: [u8; 3]) -> () {
    // NwkSKey = aes128_encrypt(AppKey, 0x01 | JoinNonce | NetID | DevNonce | pad_16)
    todo!()
}

pub fn end_device_app_skey(join_nonce: [u8; 3], net_id: [u8; 3], dev_nonce: [u8; 3]) -> () {
    // AppSKey = aes128_encrypt(AppKey, 0x02 | JoinNonce | NetID | DevNonce | pad_16)
    todo!()
}

#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct MulticastGroup {
    pub network_address: [u8; 4],
    // FIXME: not sure of the type here
    pub session_keys: (),
    pub downlink_frame_counter: u32,
}

#[cfg_attr(features = "defmt", derive(defmt::Debug))]
#[derive(Debug, Clone, Copy)]
pub struct NetworkServer<C> {
    _clock: PhantomData<C>,
}

impl<C: Clock> NetworkServer<C> {
    pub fn process_mac_request(
        &mut self,
        // TODO: consider having `message_recv_meta` and `mac_message` be the contained in the same structure
        message_recv_meta: MessageRecvMeta<C>,
        mac_message: mac::ReqFromEndDevice,
    ) -> Result<(), ()> {
        match mac_message {
            // FIXME: for `NetworkServer`, they expect to recv duplicate packets from different
            // gateways, and then place that in the alert. This suggests that don't want to send
            // the answer imediately and instead accumulate these
            mac::ReqFromEndDevice::LinkCheck => {
                // FIXME: unclear that we want to result to be returned immediately. This is at
                // it's core a queuing operation, and we won't send immediately. In the case of
                // send failure (for whatever reason) it _seems_ that we can likely ignore any
                // actual transmit errors
                //
                // FIXME: this isn't actually a command the secification says is implimented by the
                // end-device.
                self.send_mac_answer(mac::AnsFromNetworkServer::LinkCheck(mac::LinkCheckAns {
                    margin: 0,
                    gw_count: 1,
                }))
            }
        }
    }

    pub fn send_mac_answer(&mut self, _mac_answer: mac::AnsFromNetworkServer) -> Result<(), ()> {
        todo!()
    }
}
