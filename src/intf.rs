pub trait Intf {
    fn send_and_receive(&mut self, to_device: Vec<u8>) -> Vec<u8>;
    fn cmd_oneway_devicereset(&mut self, to_device: Vec<u8>);

    fn get_time_micros(&self) -> u64;
}
