//use futures_lite::future::block_on;
//use nusb::transfer::{ RequestBuffer, ControlOut, ControlType, Recipient, Queue };
//use nusb::{ Device, Interface };
//use hex_literal::hex;    //use: hex!

mod comm;
mod comm_bulk;
use crate::comm_bulk::CommBulk;



fn main() {
    println!("Hello, world!");

    env_logger::init();

    let mut comm = comm_bulk::init_comm_bulk();
    //let comm = CommBulk {};
    //comm.init();


    // set line coding request - probably not needed
    //sync_send_control(handle, 0x21, 0x20 /* set line coding*/, 0, 0, "\x00\xc2\x01\x00\x00\x00\x08", 7, 2000 );

    // NmeaSwitchCommand enable=1
    cmd_nmea_switch(&mut comm, true);

    // ModelCommand
    let model = cmd_model(&mut comm);
    println!("Model: {model}");

    // IdentificationCommand
    cmd_identification(&mut comm);

    // CountCommand
    let count = cmd_count(&mut comm);
    println!("count: {count}");

// ./decode-igotu-trace3+120b.py says ReadCommand(pos = 0x1fff80, size = 0x0008) but this is not calculatable with cpp code. I guess another impl from manufacturer
// 3411	55.575353	host	3.8.1	USB	80	URB_BULK out	930b03001d0000000000000000000042	16		CountCommand
// 3413	55.575584	3.8.1	host	USB	71	URB_BULK in		930003000b8bd4	7	
//
// 3415	55.578453	host	3.8.1	USB	80	URB_BULK out	930507000804031fff800000000000b4	16		ReadCommand (pos, size)
// 3417	55.578739	3.8.1	host	USB	76	URB_BULK in		930008ffffffffffffffff6d	12	


    let payload = cmd_read(&mut comm, 0x1fff80, 0x0008);  // from data dump of original software. no clue what is expected here // TODO force all FFs?

    if payload.len()==8 && payload==vec![0xff; 8] {
        // TODO set something. it's the time in epoc in both [s] and [ms], but for what reason?  --   usb.capdata[0] == 0x93 and usb.capdata[1] == 0x09
    } else {
        // assumption: 8xff is some signal to send this setsomething command
        panic!("Unknown device state. needs more debugging");
    }


panic!("safety stop");



    let blocks = 1 + (count + 0x7f) / 0x80;
    println!("blocks: {blocks}");
/*
            for (unsigned i = 0; i < blocks; ++i) {
                emit commandRunning(i, blocks);
                if (p->cancelRequested())
                    throw Exception(IgotuControl::tr("Cancelled"));
*/
    let i=0;
    cmd_read(&mut comm, i * 0x1000, 0x1000);

    //cmd_read(&interface, 0, 0x1000);

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
    Gt120,  // sadly, this is for both GT-120 and GT-120b
    Gt200e,
}





//==============================================================================
//==============================================================================





//==============================================================================


fn cmd_nmea_switch(comm: &mut CommBulk, _enable: bool) {
    println!("Send cmd_nmea_switch");
    let mut command : Vec<u8>= vec![0x93,0x01,0x01];

    // ignoring this: command[3] = enable ? 0x00 : 0x03;
    command.push(0x03); // 120b needs 0x03. this was the value for disabled, but it means enabled for 120b


    comm.simple_cmd_eqresult(
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


fn cmd_model(comm: &mut CommBulk) -> Model {
    println!("Send cmd_model");
    let command : Vec<u8>= vec![0x93,0x05,0x04,0x00,0x03,0x01,0x9f];


    let answer = comm.simple_cmd_return(
        command); //[0x93,0x00,0x03,0xc2,0x20,0x15,0x73].to_vec());
    /*
	ModelCommand
	3402	55.555916	host	3.8.1	USB	64	URB_BULK in						0
	3403	55.569024	host	3.8.1	USB	80	URB_BULK out	9305040003019f0000000000000000c1	16
	3404	55.569095	3.8.1	host	USB	64	URB_BULK out						0
	3405	55.569261	3.8.1	host	USB	71	URB_BULK in	930003c2201573				7
    */

    if answer[0]!=0xc2 || answer[1]!=0x20 || answer.len()!=3 {
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


fn cmd_identification(comm: &mut CommBulk) {
    println!("Send cmd_identification");
    let command : Vec<u8>= vec![0x93,0x0a];


    let answer = comm.simple_cmd_return(
        command);

    if answer.len()!=17 {
        panic!("Unexpected answer: {answer:02x?}");
    }

    let id = u32::from_be_bytes(answer[0..4].try_into().unwrap());  // was little endian in commands.cpp
    let version = u16::from_be_bytes(answer[4..6].try_into().unwrap());
    // this is far away from perfect!
    let deviceid = u16::from_be_bytes(answer[6..8].try_into().unwrap()).to_string() ;//+ "-" + hex::encode(answer[10..16]); // todo: leading zeroes and reverse order of bytes

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


fn cmd_count(comm: &mut CommBulk) -> u16 {
    println!("Send cmd_count");
    let command : Vec<u8>= vec![0x93,0x0b,0x03,0x00,0x1d];


    let answer = comm.simple_cmd_return(
        command);

    if answer.len()!=3 || answer[0]!=0x00 {
        panic!("Unexpected answer: {answer:02x?}");
    }

    let count = u16::from_be_bytes(answer[1..3].try_into().unwrap());
    println!("count: {count}");

    return count;

    /*
	CountCommand
	3410	55.570322	host	3.8.1	USB	64	URB_BULK in											0
	3411	55.575353	host	3.8.1	USB	80	URB_BULK out	930b03001d0000000000000000000042	16
	3412	55.575401	3.8.1	host	USB	64	URB_BULK out										0
	3413	55.575584	3.8.1	host	USB	71	URB_BULK in		930003000b8bd4						7

      r:Completion { data: [93, 00, 03, 00, 08, C0, A2], status: Ok(()) }
    */
}

fn cmd_read(comm: &mut CommBulk, pos: u32, size: u16) -> Vec<u8> {
    println!("Send cmd_read");
    let mut command : Vec<u8> = vec![0x93,0x05,0x07];//,0,0,0,0,0,0,0];

    println!("size: {size:x}  pos: {pos:x}");

    command.extend(&size.to_be_bytes());
    command.push(0x04);
    command.push(0x03);
    command.extend(&pos.to_be_bytes()[1..4]);
    //command[3..5]  = size.to_be_bytes();
    //command[7..10] = pos.to_be_bytes()[1..4];



    let answer = comm.simple_cmd_return(
        command);

    if answer.len()!=size as usize  {
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



//==============================================================================
//==============================================================================





