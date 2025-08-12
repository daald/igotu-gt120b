use crate::CommBulk;
use hex_literal::hex; //use: hex!

pub fn cmd_nmea_switch(comm: &mut CommBulk, _enable: bool) {
    println!("Send cmd_nmea_switch");
    let mut command: Vec<u8> = hex!["930101"].to_vec();

    // ignoring this: command[3] = enable ? 0x00 : 0x03;
    command.push(0x03); // 120b needs 0x03. this was the value for disabled, but it means enabled for 120b

    comm.simple_cmd_eqresult(command, vec![]); //[0x93,0x00,0x00,0x6d].to_vec());

    /*
    NmeaSwitchCommand
    3347	55.005277	host	3.8.1	USB	64	URB_BULK in						0
    3399	55.554806	host	3.8.1	USB	80	URB_BULK out	93010103000000000000000000000068	16
    3400	55.554842	3.8.1	host	USB	64	URB_BULK out						0
    3401	55.555664	3.8.1	host	USB	68	URB_BULK in	9300006d				4
    */
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

    /*
    ModelCommand
    3402	55.555916	host	3.8.1	USB	64	URB_BULK in						0
    3403	55.569024	host	3.8.1	USB	80	URB_BULK out	9305040003019f0000000000000000c1	16
    3404	55.569095	3.8.1	host	USB	64	URB_BULK out						0
    3405	55.569261	3.8.1	host	USB	71	URB_BULK in	930003c2201573				7
    */

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

    /*
    CountCommand
    3410	55.570322	host	3.8.1	USB	64	URB_BULK in											0
    3411	55.575353	host	3.8.1	USB	80	URB_BULK out	930b03001d0000000000000000000042	16
    3412	55.575401	3.8.1	host	USB	64	URB_BULK out										0
    3413	55.575584	3.8.1	host	USB	71	URB_BULK in		930003000b8bd4						7

      r:Completion { data: [93, 00, 03, 00, 08, C0, A2], status: Ok(()) }
    */
}

pub fn cmd_set_time(comm: &mut CommBulk, time_us: u64) {
    println!("Send cmd_set_time");
    let mut command: Vec<u8> = hex!["9309"].to_vec();

    //let time_us = 1753997870971000_u64;
    let time_s = time_us / 1_000_000;

    command.extend(&time_us.to_le_bytes()[0..8]);
    command.extend(&time_s.to_le_bytes()[0..5]);

    comm.simple_cmd_eqresult(command, vec![]);

    /*
            example:

            > 9309F8352A4F9E360600FD253E68


        3965	83.619018	host	3.7.1	USB	80	URB_BULK out	930920cdd63d9e360600da243e6800e6	16
                                                                                                        ^^^^^^^^^^ = Thursday, July 31, 2025 9:37:50 PM GMT  in [s]hex		688be22e_h = 1753997870_d [seconds]
                                                                                        ^^^^^^^^^^^^^^^^ = Thursday, July 31, 2025 9:37:50.971 PM GMT  in [us]hex
        29531	374.287401	host	3.8.1	USB	80	URB_BULK out	9309f8352a4f9e360600fd253e68001c	16
                                                                                                        ^^^^^^^^^^ = Thursday, July 31, 2025 9:38:13 PM GMT  in [s]hex
                                                                                        ^^^^^^^^^^^^^^^^ = Thursday, July 31, 2025 9:38:13.134 PM GMT  in [us]hex			063b40755b66b0_h = 1753997893134000_d [microseconds]



    actual:   [93, 09, : 00, 06, 36, 9E, 4F, 2A, 35, F8,: 00, 00, 00, 00, 68, : 7C]

            */
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

    /*
    ReadCommand::ReadCommand(DataConnection *connection, unsigned pos,
            unsigned size) :
        IgotuCommand(connection),
        size(size)
    {
        QByteArray command("\x93\x05\x07\x00\x00\x04\x03\0\0\0\0\0\0\0\0", 15);
        command[3] = (size >> 0x08) & 0xff;
        command[4] = (size >> 0x00) & 0xff;
        command[7] = (pos >> 0x10) & 0xff;
        command[8] = (pos >> 0x08) & 0xff;
        command[9] = (pos >> 0x00) & 0xff;
        setCommand(command);
    }

        ReadCommand (pos, size)
        3414	55.576110	host	3.8.1	USB	64	URB_BULK in											0
    stop at cmd:												>>>[9305071000040300000000000000004a]
        3415	55.578453	host	3.8.1	USB	80	URB_BULK out	930507000804031fff800000000000b4	16
        3416	55.578483	3.8.1	host	USB	64	URB_BULK out										0
        3417	55.578739	3.8.1	host	USB	76	URB_BULK in		930008ffffffffffffffff6d			12

        */
}

// TODO research needed
pub fn cmd_delete_reboot(comm: &mut CommBulk) {
    println!("Send cmd_delete_reboot");
    let command: Vec<u8> = hex!["9311020080"].to_vec();

    comm.simple_cmd_oneway_devicereset(command);

    /*
    > 93:11:02:00:80:00:00:00:00:00:00:00:00:00:00:da
    */
}
