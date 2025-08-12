use crate::CommBulk;
use hex_literal::hex; //use: hex!

pub fn cmd_nmea_switch(comm: &mut CommBulk, flag: bool) {
    println!("Send cmd_nmea_switch");
    let mut command: Vec<u8> = hex!["930101"].to_vec();

    command.push(if flag { 0x03 } else { 0x00 }); // 120b needs 0x03. 120 needed 0x00 (untested)

    comm.simple_cmd_eqresult(command, vec![]);
}

#[derive(strum_macros::Display, Debug, PartialEq)]
pub enum Model {
    Gt100,
    Gt200,
    Gt120, // sadly, this is for both GT-120 and GT-120b
    Gt200e,
}

pub fn cmd_model(comm: &mut CommBulk) -> Model {
    println!("Send cmd_model");
    let command: Vec<u8> = hex!["9305040003019f"].to_vec();

    let answer = comm.simple_cmd_return(command); //[0x93,0x00,0x03,0xc2,0x20,0x15,0x73].to_vec());

    if answer.len() != 3 || answer[0] != 0xc2 || answer[1] != 0x20 {
        panic!("Unexpected answer: {answer:02x?}");
    }

    let model = answer[2];
    match model {
        0x13 => return Model::Gt100,
        0x14 => return Model::Gt200,
        0x15 => return Model::Gt120, // a and b version!
        0x17 => return Model::Gt200e,
        _ => panic!("Unknown model code: {:02x}", answer[2]),
    }
}

pub fn cmd_identification(comm: &mut CommBulk) {
    println!("Send cmd_identification");
    let command: Vec<u8> = hex!["930a"].to_vec();

    let answer = comm.simple_cmd_return(command);

    if answer.len() != 17 {
        panic!("Unexpected answer: {answer:02x?}");
    }

    // TODO a lot to extract from this response
    let id = u32::from_be_bytes(answer[0..4].try_into().unwrap()); // was little endian in commands.cpp
    let version = u16::from_be_bytes(answer[4..6].try_into().unwrap());
    // this is far away from perfect!
    let deviceid = u16::from_be_bytes(answer[6..8].try_into().unwrap()).to_string(); //+ "-" + hex::encode(answer[10..16]); // todo: leading zeroes and reverse order of bytes

    println!("id: {id}  version: {version}  deviceid: {deviceid}")

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
    //                        |id----------|  |ver-|          ^ firmware?   66 0D could be device name GT120B-0D66  device id 0010-00188C710D66
    // firmware 1.2.220218 or 1.2.230111
    */
}

fn calculate_offset_from_count(b: u8, c: u8) -> u32 {
    let out_shifted = ((b as u32) << 3) + ((c as u32) >> 5) + 1;
    out_shifted << 12
}

pub fn cmd_count(comm: &mut CommBulk) -> u32 {
    println!("Send cmd_count");
    let command: Vec<u8> = hex!["930b03001d"].to_vec();

    let answer = comm.simple_cmd_return(command);

    if answer.len() != 3 {
        panic!("Unexpected answer: {answer:02x?}");
    }

    let offset = calculate_offset_from_count(answer[1], answer[2]);

    println!("count/offset: {offset}, {offset:06x}");

    return offset;
}

pub fn cmd_set_time(comm: &mut CommBulk, time_us: u64) {
    println!("Send cmd_set_time");
    let mut command: Vec<u8> = hex!["9309"].to_vec();

    //let time_us = 1753997870971000_u64;
    let time_s = time_us / 1_000_000;

    command.extend(&time_us.to_le_bytes()[0..8]);
    command.extend(&time_s.to_le_bytes()[0..5]);

    comm.simple_cmd_eqresult(command, vec![]);
}

pub fn cmd_read(comm: &mut CommBulk, pos: u32, size: u16) -> Vec<u8> {
    println!("Send cmd_read (size: {size:04x}  pos: {pos:06x}");
    let mut command: Vec<u8> = hex!["930507"].to_vec(); //,0,0,0,0,0,0,0];

    //    command[3] = (size >> 0x08) & 0xff;
    //    command[4] = (size >> 0x00) & 0xff;
    //    command[7] = (pos >> 0x10) & 0xff;
    //    command[8] = (pos >> 0x08) & 0xff;
    //    command[9] = (pos >> 0x00) & 0xff;

    command.extend(&size.to_be_bytes());
    command.push(0x04);
    command.push(0x03);
    command.extend(&pos.to_be_bytes()[1..4]);
    //command[3..5]  = size.to_be_bytes();
    //command[7..10] = pos.to_be_bytes()[1..4];

    let answer = comm.simple_cmd_return(command);

    if answer.len() != size as usize {
        panic!("Unexpected answer: {answer:02x?}");
    }
    return answer;
}

pub fn cmd_delete_reboot(comm: &mut CommBulk) {
    println!("Send cmd_delete_reboot");
    let command: Vec<u8> = hex!["9311020080"].to_vec();

    comm.simple_cmd_oneway_devicereset(command);
}
