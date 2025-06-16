use futures_lite::future::block_on;
use nusb::transfer::{ RequestBuffer, ControlOut, ControlType, Recipient, Queue };
use nusb::{ Device, Interface };
//use hex_literal::hex;    //use: hex!


const DEVID_VENDOR  :u16 = 0x0df7;
const DEVID_PRODUCT :u16 = 0x0920;
const DEVICE_INTERFACE :u8 = 1;
const BULK_EP_IN  :u8 = 0x81;
const BULK_EP_OUT :u8 = 0x01;

pub struct CommBulk {
  device: Device,
  interface: Interface,
}

  
pub fn init_comm_bulk() -> CommBulk {
    let di = nusb::list_devices()
        .unwrap()
        .find(|d| d.vendor_id() == DEVID_VENDOR && d.product_id() == DEVID_PRODUCT)
        .expect("Cannot find device");

    println!("Device info: {di:?}");

    let device = di.open().unwrap();
    let interface = device.detach_and_claim_interface(DEVICE_INTERFACE).unwrap();

    // set control line state request - needed for the device to reply in BULK mode
    //device.control_out_blocking(handle, 0x21, 0x22 /* set line state*/, 3, 0, NULL, 0, 2000);

    ctrl_set_line_state(&device);
    
    return CommBulk{device: device, interface: interface};
}

impl CommBulk {
  
  pub fn simple_cmd_return(&mut self, to_device_: Vec<u8>) -> Vec<u8> {
    let mut to_device = to_device_.clone();
    pad_and_checksum(&mut to_device);
    println!("Simple cmd {to_device:02X?}");

    let queue = self.interface.bulk_in_queue(BULK_EP_IN);

    block_on(self.interface.bulk_out(BULK_EP_OUT, to_device))
        .into_result()
        .unwrap();

    println!("  awaiting answer");
    let answer = self.read_answer(queue);
    // TODO close queue
    //println!("  r={answer:02X?}");
    return answer;
  }
  
  pub fn simple_cmd_eqresult(&mut self, to_device: Vec<u8>, expect_from_device: Vec<u8>) {
    let answer = self.simple_cmd_return(to_device);
    //println!("  r={answer:02X?}");
    check_full_answer(answer, expect_from_device);
  }

  

  
  
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

        return verify_answer_checksum_extract_payload(result.data);
    }
  }
}
  
fn ctrl_set_line_state(device: &Device) {
    println!("Send ctrl_set_line_state");
    block_on(device.control_out(ControlOut {
        control_type: ControlType::Class,
        recipient: Recipient::Device,
        request: 0x22 /* set line state*/,
        value: 0x03,
        index: 0x00,
        data: &[],
    })).into_result().unwrap();
}


fn pad_and_checksum(raw_command: &mut Vec<u8>) {
    assert!(raw_command.len() < 16);
    raw_command.resize(15, 0);
    let sum:u8=raw_command.iter().sum();
    raw_command.push(0x00 - sum);
    assert_eq!(raw_command.len(), 16);
}
  
  

fn verify_answer_checksum_extract_payload(answer: Vec<u8>) -> Vec<u8> {
    if answer[0] != 0x93 {
        panic!("Invalid prefix in answer. expected: 0x93");
    }
    let sum:u8 = answer[..answer.len()-1].iter().sum();
    let expected:u8 = 0x00 - sum;
    let actual = answer[answer.len()-1];
    if actual != expected {
        panic!("Checksum error in answer. actual: {actual:02x}, expected: {expected:02x}")
    }
    let payloadsize:u16 = u16::from_be_bytes(answer[1..3].try_into().unwrap());
    if payloadsize as u32 != (answer.len()-4) as u32 {
        panic!("Invalid playload size. declared: {payloadsize:02x}, actual: {:02x}", answer.len()-4);
    }
    return answer[3..(answer.len()-1)].to_vec();
}



fn check_full_answer(answer: Vec<u8>, expected: Vec<u8>) {
    if answer != expected {
        panic!("Wrong answer. received {answer:02X?}. expected: {expected:02X?}");
    }
    println!("all good")
}


