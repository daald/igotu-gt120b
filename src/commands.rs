use crate::CommBulk;
use hex_literal::hex;
use log::debug;
use serde::{Deserialize, Serialize}; //use: hex!

pub fn cmd_nmea_switch(comm: &mut CommBulk, flag: bool) {
    debug!("Send cmd_nmea_switch");
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
    debug!("Send cmd_model");
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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[derive(PartialEq, Debug)]
pub struct IdentificationJson {
    manufacturer: String,
    model: u16,
    #[serde(rename = "DeviceID")]
    device_id: String,
    name: String,
    pub alias: String,
    serial_number: String,
    #[serde(rename = "HWVersion")]
    hw_version: String,
    #[serde(rename = "FWVersion")]
    fw_version: String,
    #[serde(rename = "SWVersion")]
    sw_version: String,
    description: String,
}

pub fn cmd_identification(
    comm: &mut CommBulk,
    conf_orig_sw_equivalent: bool,
) -> IdentificationJson {
    debug!("Send cmd_identification");
    let command: Vec<u8> = hex!["930a"].to_vec();

    let answer = comm.simple_cmd_return(command);

    if answer.len() != 17 {
        panic!("Unexpected answer: {answer:02x?}");
    }

    let serial = u32::from_le_bytes(answer[0..4].try_into().unwrap()); // was little endian in commands.cpp
    let version1 = answer[4];
    let version2 = answer[5];
    let version3 = u16::from_le_bytes(answer[8..10].try_into().unwrap());

    let version3d = version3 & 0x1f;
    let version3m = version3 >> 5 & 0xf;
    let version3y = version3 >> 9 & 0xff;

    let name2 = format!("{:02X}{:02X}", answer[11], answer[10]);
    let version = format!("{version1}.{version2}.{version3y:02}{version3m:02}{version3d:02}");
    // this is far away from perfect!
    let model = u16::from_be_bytes(answer[6..8].try_into().unwrap()); //+ "-" + hex::encode(answer[10..16]); // todo: leading zeroes and reverse order of bytes
    let devid2 = u64::from_le_bytes(answer[8..16].try_into().unwrap()) >> 16 & 0xffffffffffff;
    let serialnumber = 10000000000u64 * model as u64 + serial as u64;
    let deviceid = format!("{model:04}-{devid2:012X}");

    let modelname = match model {
        10 => "GT120B",
        _ => panic!("Unknown model code: {model}"),
    };

    let id_struct = IdentificationJson {
        manufacturer: if conf_orig_sw_equivalent {
            "".to_owned()
        } else {
            "mobileaction //TODO".to_owned()
        },
        model: model,
        device_id: deviceid,
        name: format!("{modelname}-{name2}"), // TODO name or alias is customizable
        alias: format!("{modelname}-{name2}"),
        serial_number: serialnumber.to_string(),
        hw_version: "".to_owned(), //TODO no way to find out??
        fw_version: version,
        sw_version: "not installed".to_owned(),
        description: "".to_owned(),
    };

    println!("{}", serde_json::to_string(&id_struct).unwrap());
    println!("{}", serde_json::to_string_pretty(&id_struct).unwrap());

    return id_struct;
}

fn calculate_offset_from_count(b: u8, c: u8) -> u32 {
    let out_shifted = ((b as u32) << 3) + ((c as u32) >> 5) + 1;
    out_shifted << 12
}

pub fn cmd_count(comm: &mut CommBulk) -> u32 {
    debug!("Send cmd_count");
    let command: Vec<u8> = hex!["930b03001d"].to_vec();

    let answer = comm.simple_cmd_return(command);

    if answer.len() != 3 {
        panic!("Unexpected answer: {answer:02x?}");
    }

    let offset = calculate_offset_from_count(answer[1], answer[2]);

    debug!("count/offset: {offset}, {offset:06x}");

    return offset;
}

pub fn cmd_set_time(comm: &mut CommBulk, time_us: u64) {
    debug!("Send cmd_set_time");
    let mut command: Vec<u8> = hex!["9309"].to_vec();

    //let time_us = 1753997870971000_u64;
    let time_s = time_us / 1_000_000;

    command.extend(&time_us.to_le_bytes()[0..8]);
    command.extend(&time_s.to_le_bytes()[0..5]);

    comm.simple_cmd_eqresult(command, vec![]);
}

pub fn cmd_read(comm: &mut CommBulk, pos: u32, size: u16) -> Vec<u8> {
    debug!("Send cmd_read (size: {size:04x}  pos: {pos:06x}");
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
    debug!("Send cmd_delete_reboot");
    let command: Vec<u8> = hex!["9311020080"].to_vec();

    comm.simple_cmd_oneway_devicereset(command);
}
