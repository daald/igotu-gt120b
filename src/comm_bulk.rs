//use futures_lite::future::block_on;
//use nusb::transfer::{ RequestBuffer, ControlOut, ControlType, Recipient, Queue };
//use nusb::{ Device, Interface };
//use hex_literal::hex;    //use: hex!

//mod intf_bulk;
use crate::intf::Intf;

pub struct CommBulk {
    pub intf: Box<dyn Intf>,
}

impl CommBulk {
    pub fn simple_cmd_return(&mut self, to_device_: Vec<u8>) -> Vec<u8> {
        let mut to_device = to_device_.clone();
        pad_and_checksum(&mut to_device);
        println!("Simple cmd {to_device:02X?}");

        let answer = self.intf.send_and_receive(to_device);
        //println!("  r={answer:02X?}");
        let payload = verify_answer_checksum_extract_payload(answer);
        println!("Simple response {payload:02X?}");
        return payload;
    }

    pub fn simple_cmd_eqresult(&mut self, to_device: Vec<u8>, expect_from_device: Vec<u8>) {
        let answer = self.simple_cmd_return(to_device);
        //println!("  r={answer:02X?}");
        check_full_answer(answer, expect_from_device);
    }

    pub fn simple_cmd_oneway_devicereset(&mut self, to_device_: Vec<u8>) {
        let mut to_device = to_device_.clone();
        pad_and_checksum(&mut to_device);
        println!("Simple cmd {to_device:02X?}");

        self.intf.cmd_oneway_devicereset(to_device);
    }

    pub fn is_real(&self) -> bool {
        return self.intf.is_real();
    }

    pub fn get_time_micros(&self) -> u64 {
        return self.intf.get_time_micros();
    }
}

fn pad_and_checksum(raw_command: &mut Vec<u8>) {
    assert!(raw_command.len() < 16);
    raw_command.resize(15, 0);
    let sum: u8 = raw_command.iter().fold(0, |sum, i| sum.wrapping_add(*i));
    raw_command.push(0x00u8.wrapping_sub(sum));
    assert_eq!(raw_command.len(), 16);
}

fn verify_answer_checksum_extract_payload(answer: Vec<u8>) -> Vec<u8> {
    if answer[0] != 0x93 {
        panic!("Invalid prefix in answer. expected: 0x93");
    }
    let sum: u8 = answer[..answer.len() - 1]
        .iter()
        .fold(0, |sum, i| sum.wrapping_add(*i));
    let expected: u8 = 0x00u8.wrapping_sub(sum);
    let actual = answer[answer.len() - 1];
    if actual != expected {
        panic!("Checksum error in answer. actual: {actual:02x}, expected: {expected:02x}")
    }
    let payloadsize: u16 = u16::from_be_bytes(answer[1..3].try_into().unwrap());
    if payloadsize as u32 != (answer.len() - 4) as u32 {
        panic!(
            "Invalid playload size. declared: {payloadsize:02x}, actual: {:02x}",
            answer.len() - 4
        );
    }
    return answer[3..(answer.len() - 1)].to_vec();
}

fn check_full_answer(answer: Vec<u8>, expected: Vec<u8>) {
    if answer != expected {
        panic!("Wrong answer. received {answer:02X?}. expected: {expected:02X?}");
    }
    println!("all good")
}
