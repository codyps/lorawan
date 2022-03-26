//! Device abstraction

struct Channel {
    frequency: Frequency,
    coding_rate: CodingRate,
    modulation: Modulation,
}

trait Radio {
    fn eui(&self) -> u64;
    fn recv_enable(&mut self, channel: &Channel, recv_acceptor: RecvAcceptor) -> Result<(), ()>;
}

// `Radio` implimentors push recv'd data into this when ready
//
// In `lwip`, the `netdev` serves this purpose
struct RecvAcceptor {}
