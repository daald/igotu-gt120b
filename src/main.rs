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
    cmd_NmeaSwitch(&interface, true);

    // ModelCommand
    let model = cmd_Model(&interface);
    println!("Model: {model}");

    // IdentificationCommand
    cmd_Identification(&interface);




}




/*
    loop {
        while queue.pending() < 8 {
            queue.submit(RequestBuffer::new(256));
        }
        let result = block_on(queue.next_complete());
        println!("r:{result:02X?}");
// r:Completion { data: [147, 0, 0, 109], status: Ok(()) }
//    if (memcmp(combuf_in, "\x93\x00\x00\x6d", 4) == 0) {
//        printf("received success\n");


        if result.status.is_err() {
            break;
        }
    }
*/





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
    let payloadsize:u16 = (answer[1] as u16)<<8 | (answer[2] as u16);
    if payloadsize as u32 != (answer.len()-4) as u32 {
        panic!("Invalid playload size. declared: {payloadsize:02x}, actual: {:02x}", answer.len()-4);
    }
    return answer[3..(answer.len()-1)].to_vec();
}


fn read_answer(mut in_queue: Queue<RequestBuffer>) -> Vec<u8> {
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

fn check_full_answer(answer: Vec<u8>, expected: Vec<u8>) {
    if answer != expected {
        panic!("Wrong answer. received {answer:02X?}. expected: {expected:02X?}");
    }
    println!("all good")
}



fn padAndChecksum(rawCommand: &mut Vec<u8>) {
    assert!(rawCommand.len() < 16);
    rawCommand.resize(15, 0);
    let sum:u8=rawCommand.iter().sum();
    rawCommand.push(0x00 - sum);
    assert_eq!(rawCommand.len(), 16);
}

fn cmd_NmeaSwitch(interface: &Interface, _enable: bool) {
    let mut command = [0x93,0x01,0x01].to_vec();

    // ignoring this: command[3] = enable ? 0x00 : 0x03;
    command.push(0x03); // 120b needs 0x03. this was the value for disabled, but it means enabled for 120b

    padAndChecksum(&mut command);
    simple_cmd_eqresult(&interface,
        command,
        vec![]); //[0x93,0x00,0x00,0x6d].to_vec());
    /*
	NmeaSwitchCommand
	3347	55.005277	host	3.8.1	USB	64	URB_BULK in						0	
	3399	55.554806	host	3.8.1	USB	80	URB_BULK out	93010103000000000000000000000068	16
	3400	55.554842	3.8.1	host	USB	64	URB_BULK out						0	
	3401	55.555664	3.8.1	host	USB	68	URB_BULK in	9300006d				4	
    */
}


fn cmd_Model(interface: &Interface) -> Model {
    let mut command = [0x93,0x05,0x04,0x00,0x03,0x01,0x9f].to_vec();

    padAndChecksum(&mut command);
    let answer = simple_cmd_return(&interface,
        command); //[0x93,0x00,0x03,0xc2,0x20,0x15,0x73].to_vec());
    /*
	ModelCommand
	3402	55.555916	host	3.8.1	USB	64	URB_BULK in						0	
	3403	55.569024	host	3.8.1	USB	80	URB_BULK out	9305040003019f0000000000000000c1	16
	3404	55.569095	3.8.1	host	USB	64	URB_BULK out						0	
	3405	55.569261	3.8.1	host	USB	71	URB_BULK in	930003c2201573				7	
    */

    if answer[0]!=0xc2 || answer[1]!=0x20 {
        panic!("Unexpected answer: {answer:02x?}");
    }

    let model = answer[2];
    match model{
        0x13 => return Model::Gt100,
        0x14 => return Model::Gt200,
        0x15 => return Model::Gt120,
        0x17 => return Model::Gt200e,
        _ => panic!("Unknown model: {:02x}", answer[2]),
    }
}

#[derive(strum_macros::Display)]
enum Model {
    Gt100,
    Gt200,
    Gt120,  // also for 120b
    Gt200e,
}


fn cmd_Identification(interface: &Interface) {
    let mut command = [0x93,0x0a].to_vec();

    padAndChecksum(&mut command);
    simple_cmd_eqresult(&interface,
        command,
        hex!("930011a623630d0102000a2b2e660d718c18000233").to_vec());
    /*
	IdentificationCommand
	3406	55.569581	host	3.8.1	USB	64	URB_BULK in						0	
	3407	55.569800	host	3.8.1	USB	80	URB_BULK out	930a0000000000000000000000000063	16
	3408	55.569849	3.8.1	host	USB	64	URB_BULK out						0	
	3409	55.570052	3.8.1	host	USB	85	URB_BULK in	930011a623630d0102000a2b2e660d718c18000233	21	

    simple_cmd(&interface,
        [0x93,0x0a,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x63].to_vec(),
        hex!("930011a623630d0102000a2b2e660d718c18000233").to_vec()); //unknown

    //  received [93, 00, 11, A6, 23, 63, 0D, 01, 02, 00, 0A, 4D, 2F, 66, 0D, 71, 8C, 18, 00, 02, 10]
    // expected: [93, 00, 11, A6, 23, 63, 0D, 01, 02, 00, 0A, 2B, 2E, 66, 0D, 71, 8C, 18, 00, 02, 33]
    //                                                        ^^  ^^                              ^^
    */
}



fn simple_cmd_eqresult(interface: &Interface, to_device: Vec<u8>, expect_from_device: Vec<u8>) {
    println!("Simple cmd {to_device:02X?}");

    let queue = interface.bulk_in_queue(BULK_EP_IN);

    block_on(interface.bulk_out(BULK_EP_OUT, to_device))
        .into_result()
        .unwrap();

    println!("  awaiting answer");
    let answer = read_answer(queue);
    //println!("  r={answer:02X?}");
    check_full_answer(answer, expect_from_device);
}

fn simple_cmd_return(interface: &Interface, to_device: Vec<u8>) -> Vec<u8> {
    println!("Simple cmd {to_device:02X?}");

    let queue = interface.bulk_in_queue(BULK_EP_IN);

    block_on(interface.bulk_out(BULK_EP_OUT, to_device))
        .into_result()
        .unwrap();

    println!("  awaiting answer");
    let answer = read_answer(queue);
    //println!("  r={answer:02X?}");
    return answer;
}
