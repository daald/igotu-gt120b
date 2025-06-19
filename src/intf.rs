pub trait Intf {
  fn send_and_receive(&mut self, to_device: Vec<u8>) -> Vec<u8>;
  fn is_real(&self) -> bool;
}
