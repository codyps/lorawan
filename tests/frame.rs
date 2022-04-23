/// LoRaWAN® L2 1.0.4 Specification
/// 13.4 Beacon Encoding Examples
///
/// 00 00 | 00 00 02 CC | A2 7E | 00 | 01 20 00 | 00 81 03 | DE 55
///
/// Field     | RFU | Param | Time     | CRC  | InfoDesc | Lat    | Lng    | CRC  |
/// Value Hex | 00  | 00    | CC020000 | 7EA2 | 0        | 002001 | 038100 | 55DE |
#[test]
fn beacon_eu868_sf_9() {
    let beacon = [
        0x00, 0x00, 0x00, 0x00, 0x02, 0xCC, 0xA2, 0x7E, 0x00, 0x01, 0x20, 0x00, 0x00, 0x81, 0x03,
        0xDE, 0x55,
    ];

    let v = lorawan::Beacon::parse_beacon(lorawan::Sf::_9, &beacon)
        .expect("beacon did not parse properly");
}

/// LoRaWAN® L2 1.0.4 Specification
/// 13.4 Beacon Encoding Examples
///
/// 00 00 00 | 00 00 02 CC | A2 7E | 00 | 01 20 00 | 00 81 03 | 00 | 50 D4
///
/// NOTE: the 1.0.4 spec mislabels this as SF12, but the rfu sizes are consistent with SF10
#[test]
fn beacon_us915_sf_10() {
    let beacon = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0xCC, 0xA2, 0x7E, 0x00, 0x01, 0x20, 0x00, 0x00, 0x81,
        0x03, 0x00, 0x50, 0xD4,
    ];

    let v = lorawan::Beacon::parse_beacon(lorawan::Sf::_10, &beacon)
        .expect("beacon did not parse properly");
}
