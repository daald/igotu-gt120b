mod comm_bulk;
mod commands;
mod intf;
mod intf_bulk;
mod intf_file;
use crate::comm_bulk::CommBulk;
use crate::commands::{
    cmd_count, cmd_delete_reboot, cmd_identification, cmd_model, cmd_nmea_switch, cmd_read,
    cmd_set_time,
};
use crate::intf::Intf;
use crate::intf_bulk::IntfBulk;
use crate::intf_file::IntfFile;
use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Use a real device instead of a simulation
    #[arg(short, long, default_value_t = false)]
    real: bool,

    /// Number of times to greet
    //#[arg(short, long, default_value_t = 1)]
    //count: u8,

    /// Run some more commands to match replay file
    #[arg(short, long, default_value_t = false)]
    bestreplay: bool,

    /// Filename of simulation replay file
    #[arg(short, long)]
    sim_file_name: String,
}

fn main() {
    let args = Args::parse();

    env_logger::init();

    //dbg!(&args);

    let intf: Box<dyn Intf> = if args.real {
        Box::new(IntfBulk::new())
    } else {
        Box::new(IntfFile::new(args.sim_file_name))
    };
    let mut comm = CommBulk { intf: intf };
    //let comm = CommBulk {};
    //comm.init();

    // set line coding request - probably not needed
    //sync_send_control(handle, 0x21, 0x20 /* set line coding*/, 0, 0, "\x00\xc2\x01\x00\x00\x00\x08", 7, 2000 );

    let (id_count, id_offset) = cmdblock_identify(&mut comm);

    // ./decode-igotu-trace3+120b.py says ReadCommand(pos = 0x1fff80, size = 0x0008) but this is not calculatable with cpp code. I guess another impl from manufacturer
    // 3411	55.575353	host	3.8.1	USB	80	URB_BULK out	930b03001d0000000000000000000042	16		CountCommand
    // 3413	55.575584	3.8.1	host	USB	71	URB_BULK in		930003000b8bd4	7
    //
    // 3415	55.578453	host	3.8.1	USB	80	URB_BULK out	930507000804031fff800000000000b4	16		ReadCommand (pos, size)
    // 3417	55.578739	3.8.1	host	USB	76	URB_BULK in		930008ffffffffffffffff6d	12

    let id_read = cmd_read(&mut comm, 0x1fff80, 0x0008); // from data dump of original software. no clue what is expected here // TODO force all FFs?

    if id_read.len() == 8 && id_read == vec![0xff; 8] {
        // TODO set something. it's the time in epoc in both [s] and [us], but for what reason?  --   usb.capdata[0] == 0x93 and usb.capdata[1] == 0x09
        let time_us = comm.get_time_micros();
        cmd_set_time(&mut comm, time_us); //  1753997870971000_u64

    //> 93:09:20:cd:d6:3d:9e:36:06:00:da:24:3e:68:00:e6  or 93:09:b0:cd:7f:a0:39360600d28c37680056
    //< 93:00:00:6d
    } else {
        // possibly this non-empty information is important. maybe a bad block list? fortunately or unfortunately, I've never seen this
        panic!("Unknown device state. needs more debugging");
    }

    if args.bestreplay {
        // run "./cargo-run.sh --bestreplay" for a complete run of the replay file

        // same again? at least check that the two results are squal
        let (count2, offset2) = cmd_count(&mut comm);
        assert_eq!(id_count, count2);
        assert_eq!(id_offset, offset2);
        let read_payload2 = cmd_read(&mut comm, 0x1fff80, 0x0008); // from data dump of original software. no clue what is expected here // TODO force all FFs?
        assert_eq!(id_read, read_payload2);
    }

    cmd_read(&mut comm, 0x000000, 0x00ea); // from data dump of original software. no clue why these offsets/sizes

    {
        let (count, offset) = cmd_count(&mut comm);
        assert_eq!(id_count, count);
        assert_eq!(id_offset, offset);
    }

    let (end_offset, all_begin_empty) = cmdblock_find_end_offset(&mut comm, id_offset);

    println!("A2");

    let mut offset = 0x1000;
    {
        while offset < end_offset {
            cmdblock_read_doublet(&mut comm, offset);
            offset += 0x1000;
        }
    }
    println!("B {id_offset:06x} {end_offset:06x} {offset:06x}");
    println!("B");

    if !all_begin_empty {
        cmd_read(&mut comm, offset + 0x000000, 0x0100);
        cmd_read(&mut comm, offset + 0x000f80, 0x0080);
        cmd_read(&mut comm, offset + 0x000100, 0x0e80);
    }

    if comm.is_real() {
        panic!("safety stop");
    }

    cmd_delete_reboot(&mut comm);

    // here: device reboots itself without returning an answer

    let (_id2_count, _id2_offset) = cmdblock_identify(&mut comm);
    //assert_eq!(id_count, id2_count); // TODO verify model, serial etc. count WILL be different

    let payload = cmd_read(&mut comm, 0x1fff80, 0x0008); // from data dump of original software. no clue what is expected here // TODO force all FFs?
    assert!(
        payload.len() == 8 && payload == vec![0xff; 8],
        "Unknown device state. needs more debugging"
    );

    let time_us = comm.get_time_micros();
    cmd_set_time(&mut comm, time_us); //  1753997893134000u64

    println!("END");
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
            r0 = cmdblock_read_doublet(comm, id_offset + i * 0x1000);
            if r0 {
                end_offset = id_offset + i * 0x1000;
                all_begin_empty = false;
            }
            i += 1;
        }

        println!("A1");

        cmd_read(comm, id_offset + (i - 1) * 0x1000 + 0xf80, 0x080); // from data dump of original software. no clue
    }
    return (end_offset, all_begin_empty);
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

#[derive(strum_macros::Display)]
enum Model {
    Gt100,
    Gt200,
    Gt120, // sadly, this is for both GT-120 and GT-120b
    Gt200e,
}

//==============================================================================
//==============================================================================

//==============================================================================

fn cmdblock_identify(comm: &mut CommBulk) -> (u32, u32) {
    println!("In cmdblock_identify()");

    // NmeaSwitchCommand enable=1
    cmd_nmea_switch(comm, true);

    // ModelCommand
    let model = cmd_model(comm);
    println!("Model: {model}");

    // IdentificationCommand
    cmd_identification(comm);

    // CountCommand
    let (count, offset) = cmd_count(comm);

    //TODO return all identification results
    return (count, offset);
}

fn cmdblock_read_doublet(comm: &mut CommBulk, pos: u32) -> bool {
    let resp1 = cmd_read(comm, pos + 0x000000, 0x0100); // beginning. also used for probing
    if resp1 == vec![0xff; 0x0100] {
        println!("skip 2nd read");
        return false;
    }
    let _resp2 = cmd_read(comm, pos + 0x000100, 0x0f00); // rest
    return true;
}

//==============================================================================
//==============================================================================
