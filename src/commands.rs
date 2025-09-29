use crate::CommBulk;
use hex_literal::hex;
use log::debug;
use serde::{Deserialize, Serialize};

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

    let answer = comm.simple_cmd_return(command);

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
    let model = u16::from_be_bytes(answer[6..8].try_into().unwrap());
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
        // TODO downloader-version (this version) "igotu-gt120 1.2.3/linux https://github/link"
    };

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

    let time_s = time_us / 1_000_000;

    command.extend(&time_us.to_le_bytes()[0..8]);
    command.extend(&time_s.to_le_bytes()[0..5]);

    comm.simple_cmd_eqresult(command, vec![]);
}

pub fn cmd_read(comm: &mut CommBulk, pos: u32, size: u16) -> Vec<u8> {
    debug!("Send cmd_read (size: {size:04x}  pos: {pos:06x}");
    let mut command: Vec<u8> = hex!["930507"].to_vec();

    command.extend(&size.to_be_bytes());
    command.push(0x04);
    command.push(0x03);
    command.extend(&pos.to_be_bytes()[1..4]);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Intf;

    struct IntfMock {
        req: Vec<u8>,
        res: Vec<u8>,
    }

    fn new_mock(req: Vec<u8>, res: Vec<u8>) -> CommBulk {
        CommBulk::new(Box::new(IntfMock {
            req: req.to_vec(),
            res: res.to_vec(),
        }))
    }

    impl Intf for IntfMock {
        fn send_and_receive(&mut self, to_device: Vec<u8>) -> Vec<u8> {
            assert_eq!(to_device, self.req);
            return self.res.clone();
        }

        fn cmd_oneway_devicereset(&mut self, to_device: Vec<u8>) {
            assert_eq!(to_device, self.req);
            assert_eq!(self.res, Vec::<u8>::new());
        }

        fn get_time_micros(&self) -> u64 {
            panic!("Not implemented");
        }
    }

    #[test]
    fn cmd_nmea_switch_goodcase() {
        let mut comm = new_mock(
            hex!["93 01 01 03 00 00 00 00 00 00 00 00 00 00 00 68"].to_vec(),
            hex!["93 00 00 6d"].to_vec(),
        );
        let flag = true;

        cmd_nmea_switch(&mut comm, flag);
    }

    #[test]
    fn cmd_model_goodcase() {
        let mut comm = new_mock(
            hex!["93 05 04 00 03 01 9f 00 00 00 00 00 00 00 00 c1"].to_vec(),
            hex!["93 00 03 c2 20 15 73"].to_vec(),
        );

        let result = cmd_model(&mut comm);

        assert_eq!(result, Model::Gt120);
    }

    #[test]
    fn cmd_identification_true_goodcase() {
        let mut comm = new_mock(
            hex!["93 0a 00 00 00 00 00 00 00 00 00 00 00 00 00 63"].to_vec(),
            hex!["93 00 11 a6 23 63 0d 01 02 00 0a 4d 2f 66 0d 71 8c 18 00 02 10"].to_vec(),
        );

        let result = cmd_identification(&mut comm, true);

        assert_eq!(result.manufacturer, "");
        assert_eq!(result.model, 10);
        assert_eq!(result.device_id, "0010-00188C710D66");
        assert_eq!(result.name, "GT120B-0D66");
        assert_eq!(result.alias, "GT120B-0D66");
        assert_eq!(result.serial_number, "100224600998");
        assert_eq!(result.hw_version, "");
        assert_eq!(result.fw_version, "1.2.231013");
        assert_eq!(result.sw_version, "not installed");
        assert_eq!(result.description, "");
    }

    #[test]
    fn cmd_count_goodcase() {
        let mut comm = new_mock(
            hex!["93 0b 03 00 1d 00 00 00 00 00 00 00 00 00 00 42"].to_vec(),
            hex!["93 00 03 00 0f 2b 30"].to_vec(),
        );

        let result = cmd_count(&mut comm);

        assert_eq!(result, 0x7A000);
    }

    #[test]
    fn cmd_set_time_goodcase() {
        let mut comm = new_mock(
            hex!["93 09 78 38 09 74 40 3b 06 00 2e e2 8b 68 00 b3"].to_vec(),
            hex!["93 00 00 6d"].to_vec(),
        );

        let time_us = 1753997870971000_u64;

        cmd_set_time(&mut comm, time_us);
    }

    #[test]
    fn cmd_read_goodcase() {
        let mut comm = new_mock(
            hex!["93 05 07 00 08 04 03 1f ff 80 00 00 00 00 00 b4"].to_vec(),
            hex!["93 00 08 11 22 3f 44 55 66 77 88 f5"].to_vec(),
        );

        let size = 0x0008;
        let pos = 0x1fff80;
        let result = cmd_read(&mut comm, pos, size);

        assert_eq!(result, hex!["11 22 3f 44 55 66 77 88"].to_vec());
    }

    #[test]
    fn cmd_delete_reboot_goodcase() {
        let mut comm = new_mock(
            hex!["93 11 02 00 80 00 00 00 00 00 00 00 00 00 00 da"].to_vec(),
            hex![""].to_vec(),
        );

        cmd_delete_reboot(&mut comm);
    }
}
