use futures_lite::future::block_on;
use nusb::transfer::{ RequestBuffer, ControlOut, ControlType, Recipient, Queue };
use nusb::{ Interface };
use hex_literal::hex;


const DEVID_VENDOR  :u16 = 0x0df7;
const DEVID_PRODUCT :u16 = 0x0920;
const DEVICE_INTERFACE :u8 = 1;
const BULK_EP_IN  :u8 = 0x81;
const BULK_EP_OUT :u8 = 0x01;

fn main() {
    println!("Hello, world!");

    env_logger::init();
    let di = nusb::list_devices()
        .unwrap()
        .find(|d| d.vendor_id() == DEVID_VENDOR && d.product_id() == DEVID_PRODUCT)
        .expect("Cannot find device");

    println!("Device info: {di:?}");

    let device = di.open().unwrap();
    let interface = device.detach_and_claim_interface(DEVICE_INTERFACE).unwrap();

    // set control line state request - needed for the device to reply in BULK mode
    //device.control_out_blocking(handle, 0x21, 0x22 /* set line state*/, 3, 0, NULL, 0, 2000);

block_on(device.control_out(ControlOut {
    control_type: ControlType::Class,
    recipient: Recipient::Device,
    request: 0x22 /* set line state*/,
    value: 0x03,
    index: 0x00,
    data: &[],
})).into_result().unwrap();

    println!("sent control");


    // set line coding request - probably not needed
    //sync_send_control(handle, 0x21, 0x20 /* set line coding*/, 0, 0, "\x00\xc2\x01\x00\x00\x00\x08", 7, 2000 );

    // NmeaSwitchCommand enable=1
    simple_cmd(&interface,
        [0x93,0x01,0x01,0x03,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x68].to_vec(),
        [0x93,0x00,0x00,0x6d].to_vec());

    // ModelCommand
    simple_cmd(&interface,
        [0x93,0x05,0x04,0x00,0x03,0x01,0x9f,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0xc1].to_vec(),
        [0x93,0x00,0x03,0xc2,0x20,0x15,0x73].to_vec());

    // IdentificationCommand
    simple_cmd(&interface,
        [0x93,0x0a,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x63].to_vec(),
        hex!("930011a623630d0102000a4d2f660d718c18000210").to_vec()); //unknown




}




/*
    loop {
        while queue.pending() < 8 {
            queue.submit(RequestBuffer::new(256));
        }
        let result = block_on(queue.next_complete());
        println!("r:{result:?}");
// r:Completion { data: [147, 0, 0, 109], status: Ok(()) }
//    if (memcmp(combuf_in, "\x93\x00\x00\x6d", 4) == 0) {
//        printf("received success\n");


        if result.status.is_err() {
            break;
        }
    }
*/







fn read_answer(mut in_queue: Queue<RequestBuffer>) -> Vec<u8> {
    loop {
        while in_queue.pending() < 8 {
            in_queue.submit(RequestBuffer::new(256));
        }
        let result = block_on(in_queue.next_complete());
        println!("  r:{result:?}");
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

fn check_full_answer(answer: Vec<u8>, expected: Vec<u8>) {
    if answer != expected {
        panic!("Wrong answer. received {answer:?}. expected: {expected:?}");
    }
    println!("all good")
}

fn simple_cmd(interface: &Interface, to_device: Vec<u8>, expect_from_device: Vec<u8>) {
    println!("Simple cmd");

    let queue = interface.bulk_in_queue(BULK_EP_IN);

    block_on(interface.bulk_out(BULK_EP_OUT, to_device))
        .into_result()
        .unwrap();

    println!("  awaiting answer");
    let answer = read_answer(queue);
    //println!("  r={answer:?}");
    check_full_answer(answer, expect_from_device)
}