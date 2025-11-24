use crate::comm_bulk::CommBulk;
use crate::commands::IdentificationJson;
use crate::commands::{
    Model, cmd_count, cmd_delete_reboot, cmd_identification, cmd_model, cmd_nmea_switch, cmd_read,
    cmd_set_time,
};
use crate::gt120b_datadump::Gt120bDataDump;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use log::{debug, info, trace};

pub fn workflow(
    comm: &mut CommBulk,
    conf_clear: bool,
    conf_orig_sw_workflow: bool,
    conf_orig_sw_meta: bool,
    conf_prefix: String,
    conf_suffix: String,
) {
    // set line coding request - probably not needed
    //sync_send_control(handle, 0x21, 0x20 /* set line coding*/, 0, 0, "\x00\xc2\x01\x00\x00\x00\x08", 7, 2000 );

    let (id_model, id_offset, mut id_struct) = cmdblock_identify(comm, conf_orig_sw_meta);
    assert_eq!(id_model, Model::Gt120);

    let read8_payload = cmd_read(comm, 0x1fff80, 0x0008); // from data dump of original software. no clue what is expected here // TODO force all FFs?
    if read8_payload.len() == 8 && read8_payload == vec![0xff; 8] {
        // I don't really know why the time is sent here, but the original sw does too
        let time_us = comm.get_time_micros();
        cmd_set_time(comm, time_us); //  1753997870971000_u64
    } else {
        // possibly this non-empty information is important. maybe a bad block list? fortunately or unfortunately, I've never seen this case
        panic!("Unknown device state. needs more debugging/development");
    }

    if conf_orig_sw_workflow {
        // this block was introduced because the original sw does these calls, and I want to have a 100% identical replay for quality reasons.
        // but actually, I don't know what is done here and why. maybe it's an artifact of the incremental algorighm of the original software
        // (if you don't delete your data, already loaded data get skipped on next read, with the help of a local state storage)

        // we don't know what to do, but at least check that the results match to what was before
        let offset2 = cmd_count(comm);
        assert_eq!(id_offset, offset2);
        let read8_payload2 = cmd_read(comm, 0x1fff80, 0x0008);
        assert_eq!(read8_payload, read8_payload2);
    }

    cmdblock_readconfig(comm, &mut id_struct);

    {
        let offset = cmd_count(comm);
        assert_eq!(id_offset, offset);
    }

    let (end_offset, all_begin_empty) = cmdblock_find_end_offset(comm, id_offset);

    info!("Start downloading data");
    let mut datadumper = Gt120bDataDump::new(conf_prefix, conf_suffix);
    let mut datadumper_ref = Some(&mut datadumper);
    let mut offset = 0x1000;
    while offset < end_offset {
        cmdblock_read_doublet(comm, offset, &mut datadumper_ref);
        offset += 0x1000;
    }
    trace!("offsets: {id_offset:06x} {end_offset:06x} {offset:06x}");

    if !all_begin_empty {
        // result is important in some usecases
        let resp = cmd_read(comm, offset, 0x0100);
        if let Some(ref mut datadumper) = datadumper_ref {
            datadumper.process_datablock(resp);
        }
        let resp = cmd_read(comm, offset + 0x000f80, 0x0080);
        if let Some(ref mut datadumper) = datadumper_ref {
            datadumper.process_datablock(resp);
        }
        let resp = cmd_read(comm, offset + 0x000100, 0x0e80);
        if let Some(ref mut datadumper) = datadumper_ref {
            datadumper.process_datablock(resp);
        }
    }

    info!("Dumping to GPX");

    if let Some(ref mut datadumper) = datadumper_ref {
        let conf_change_every_day: bool = true;
        let meta_desc = if conf_orig_sw_meta {
            let json_str_compact = serde_json::to_string(&id_struct).unwrap();
            BASE64_STANDARD.encode(json_str_compact)
        } else {
            serde_json::to_string(&id_struct).unwrap() // TODO formatted output
        };
        let num_files = datadumper
            .write_out(conf_change_every_day, &meta_desc)
            .expect("Problem while exporting to gpx files");
        if num_files == 0 {
            // stopping here, there was nothing saved, so there's nothing to delete
            return;
        }
    }

    if !conf_clear {
        // stopping here, rest is only for deleting
        return;
    }

    info!("Delete device data");
    cmd_delete_reboot(comm);

    // here: device reboots itself without returning an answer. not that it will disconnect and needs to be reconnected afterwards for making sure the delete was successful
    info!("Waiting for device reconnect");

    let (id2_model, _id2_offset, id2_struct) = cmdblock_identify(comm, conf_orig_sw_meta);
    // check everything except offset
    assert_eq!(id_model, id2_model);
    id_struct.alias = id2_struct.alias.clone(); // fix value for comparing in the following line
    assert_eq!(id_struct, id2_struct);

    let payload = cmd_read(comm, 0x1fff80, 0x0008); // from data dump of original software. no clue what is expected here // TODO force all FFs?
    assert!(
        payload.len() == 8 && payload == vec![0xff; 8],
        "Unknown device state. needs more debugging"
    );

    let time_us = comm.get_time_micros();
    cmd_set_time(comm, time_us);
}

fn cmdblock_readconfig(comm: &mut CommBulk, id_struct: &mut IdentificationJson) {
    let name_config_response = cmd_read(comm, 0x000000, 0x00ea);

    let name = String::from_utf8_lossy(&name_config_response[16..48]); // is utf-8
    let name = name.trim_end_matches('\0');
    id_struct.alias = name.to_string();
    println!("NAME: <{name}> {}", name.len());
    println!("< {name_config_response:X?}");
    println!("CONFIG: normal interval: {}s", name_config_response[4]);
    println!(
        "CONFIG: smart tracking above {}kmh: {}s",
        name_config_response[2/*or 11*/], name_config_response[8]
    );
    //TODO there are some other values in this response:
    //< 10:0e
    //< 19:00:38:00:07:00:00:02
    //< f0:a0:90:65:76:7b:91:65
    //< 01:d8:ff:04:01:06:09:21:20:f5
}

fn cmdblock_find_end_offset(comm: &mut CommBulk, id_offset: u32) -> (u32, bool) {
    let mut end_offset = id_offset;
    let mut all_begin_empty = true;
    {
        let mut r1 = false;
        let mut r0 = false;
        let mut i = 0;
        while i < 2 || r0 || r1 {
            r1 = r0;
            r0 = cmdblock_read_doublet(comm, id_offset + i * 0x1000, &mut None); // TODO maybe also datadump here. we don't want to lose anything, be I also know we read these blocks multiple times
            if r0 {
                end_offset = id_offset + i * 0x1000;
                all_begin_empty = false;
            }
            i += 1;
        }

        cmd_read(comm, id_offset + (i - 1) * 0x1000 + 0xf80, 0x080); // from data dump of original software. no clue
    }
    (end_offset, all_begin_empty)
}

fn cmdblock_identify(
    comm: &mut CommBulk,
    conf_orig_sw_meta: bool,
) -> (Model, u32, IdentificationJson) {
    debug!("cmdblock_identify()");

    // NmeaSwitchCommand enable=1
    cmd_nmea_switch(comm, true);

    // ModelCommand
    let model = cmd_model(comm);
    println!("Model: {model}");

    // IdentificationCommand
    let id_struct = cmd_identification(comm, conf_orig_sw_meta);

    // CountCommand
    let offset = cmd_count(comm);

    (model, offset, id_struct)
}

/*
 * Seen in original software: Read 0x100 bytes first, and then more if they were not all == 0xFF
 */
fn cmdblock_read_doublet(
    comm: &mut CommBulk,
    pos: u32,
    datadumper_ref: &mut Option<&mut Gt120bDataDump>,
) -> bool {
    let resp1 = cmd_read(comm, pos, 0x0100); // beginning. also used for probing
    if resp1 == vec![0xff; 0x0100] {
        trace!("empty block. skip 2nd read");
        return false;
    }

    if let Some(datadumper) = datadumper_ref {
        datadumper.process_datablock(resp1);
    }
    let resp2 = cmd_read(comm, pos + 0x000100, 0x0f00); // rest
    if let Some(datadumper) = datadumper_ref {
        datadumper.process_datablock(resp2);
    }
    true
}
