// example from https://github.com/anthonykirby/lora-packet/blob/master/README.md
#[test]
fn lora_aa() {
    let buf = hex::decode("40F17DBE4900020001954378762B11FF0D").unwrap();

    let _pkt = lorawan::mac_frame::PhyPayload::from_bytes(&buf[..]).unwrap();

    let _nw_skey = hex::decode("44024241ed4ce9a68c6a8bc055233fd3").unwrap();
    let _app_skey = hex::decode("ec925802ae430ca77fd3dd73cb2cc588").unwrap();
}

// https://lorawan-packet-decoder-0ta6puiniaut.runkit.sh/?data=40AE130426800000016F895D98810714E3268295&nwkskey=99D58493D1205B43EFF938F0F66C339E&appskey=0A501524F8EA5FCBF9BDB5AD7D126F75
#[test]
fn lora_2() {
    let buf = hex::decode("40AE130426800000016F895D98810714E3268295").unwrap();

    let _pkt = lorawan::mac_frame::PhyPayload::from_bytes(&buf[..]).unwrap();

    let _nw_skey = hex::decode("99D58493D1205B43EFF938F0F66C339E").unwrap();
    let _app_skey = hex::decode("0A501524F8EA5FCBF9BDB5AD7D126F75").unwrap();
}

// example from https://lorawan-packet-decoder-0ta6puiniaut.runkit.sh/
// https://lorawan-packet-decoder-0ta6puiniaut.runkit.sh/?data=ANwAANB%2B1bNwHm/t9XzurwDIhgMK8sk=&appskey=B6B53F4A168A7A88BDF7EA135CE9CFCA
#[test]
fn join_request() {
    let buf = hex::decode("00DC0000D07ED5B3701E6FEDF57CEEAF00C886030AF2C9").unwrap();

    let pkt = lorawan::mac_frame::PhyPayload::from_bytes(&buf[..]).unwrap();

    let app_key = hex::decode("B6B53F4A168A7A88BDF7EA135CE9CFCA").unwrap();

    assert_eq!(
        pkt.mac_header(),
        lorawan::mac_frame::MacHeader::new().with_ftype(lorawan::mac_frame::FrameType::JoinRequest)
    );

    assert_eq!(u32::from_be_bytes(pkt.mic()), 0x030AF2C9);

    assert_eq!(u32::from_be_bytes(pkt.mic_expected(&app_key)), 0x030AF2C9);

    let _payload = if let lorawan::mac_frame::Payload::JoinRequest(a) = pkt.payload().unwrap() {
        a
    } else {
        panic!()
    };
}
