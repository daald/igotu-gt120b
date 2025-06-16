use futures_lite::future::block_on;
use nusb::transfer::{ RequestBuffer, ControlOut, ControlType, Recipient, Queue };
use nusb::{ Device, Interface };
//use hex_literal::hex;    //use: hex!

#[path = "intf.rs"]
mod intf;
pub use intf::Intf;




const DEVID_VENDOR  :u16 = 0x0df7;
const DEVID_PRODUCT :u16 = 0x0920;
const DEVICE_INTERFACE :u8 = 1;
const BULK_EP_IN  :u8 = 0x81;
const BULK_EP_OUT :u8 = 0x01;


pub struct IntfBulk {
  device: Device,
  interface: Interface,
}

pub fn init_intf_bulk() -> IntfBulk {
    let di = nusb::list_devices()
        .unwrap()
        .find(|d| d.vendor_id() == DEVID_VENDOR && d.product_id() == DEVID_PRODUCT)
        .expect("Cannot find device");

    println!("Device info: {di:?}");

    let device = di.open().unwrap();
    let interface = device.detach_and_claim_interface(DEVICE_INTERFACE).unwrap();

    // set control line state request - needed for the device to reply in BULK mode
    //device.control_out_blocking(handle, 0x21, 0x22 /* set line state*/, 3, 0, NULL, 0, 2000);

    let mut intf = IntfBulk{device: device, interface: interface};
    intf.ctrl_set_line_state();

    return intf;
}

impl Intf for IntfBulk {
  fn send_and_receive(&mut self, to_device: Vec<u8>) -> Vec<u8> {
    let queue = self.interface.bulk_in_queue(BULK_EP_IN);

    block_on(self.interface.bulk_out(BULK_EP_OUT, to_device))
        .into_result()
        .unwrap();

    println!("  awaiting answer");
    let answer = self.read_answer(queue);
    // TODO close queue
    return answer;
  }
}

impl IntfBulk {

  fn read_answer(&mut self, mut in_queue: Queue<RequestBuffer>) -> Vec<u8> {
    loop {
        while in_queue.pending() < 8 {
            in_queue.submit(RequestBuffer::new(256));
        }
        let result = block_on(in_queue.next_complete());
        println!("  r:{result:02X?}");
// r:Completion { data: [147, 0, 0, 109], status: Ok(()) }
//    if (memcmp(combuf_in, "\x93\x00\x00\x6d", 4) == 0) {
//        printf("received success\n");


        if result.status.is_err() {
            panic!("error result");
            //break;
        }

        return result.data;
    }
  }
  
  fn ctrl_set_line_state(&mut self) {
    println!("Send ctrl_set_line_state");
    block_on(self.device.control_out(ControlOut {
        control_type: ControlType::Class,
        recipient: Recipient::Device,
        request: 0x22 /* set line state*/,
        value: 0x03,
        index: 0x00,
        data: &[],
    })).into_result().unwrap();
  }

}

