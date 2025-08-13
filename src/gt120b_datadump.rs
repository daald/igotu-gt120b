use chrono::TimeZone;
use chrono::Utc;

pub struct Gt120bDataDump {
    //device: Device,
    //interface: Interface,
}

/*
impl Intf for IntfBulk {
    fn send_and_receive(&mut self, to_device: Vec<u8>) -> Vec<u8> {
        let queue = self.interface.bulk_in_queue(BULK_EP_IN);

        block_on(self.interface.bulk_out(BULK_EP_OUT, to_device))
            .into_result()
            .unwrap();

        println!("  awaiting answer");
        let answer = self.read_answer(queue);
        // TODO close queue
        return answer;
    }

    fn cmd_oneway_devicereset(&mut self, to_device: Vec<u8>) {
        block_on(self.interface.bulk_out(BULK_EP_OUT, to_device))
            .into_result()
            .unwrap();

        println!("  TODO: wait for device reset");
    }

    fn is_real(&self) -> bool {
        return true;
    }

    fn get_time_micros(&self) -> u64 {
        let duration_since_epoch = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let timestamp_micros = duration_since_epoch.as_micros();
        return timestamp_micros as u64;
    }
}
*/

impl Gt120bDataDump {
    pub fn process_datablock(&self, data: Vec<u8>) {
        let structsize = 8 + 4 * 30;
        assert_eq!(0, data.len() % structsize);
        dumpblock_hex(data);
        //       dumpblock_parse(data);
    }
}

fn dumpblock_hex(data: Vec<u8>) {
    // TODO print offset for verbosity
    let mut pos = 0;
    while pos < data.len() {
        println!("Received data: {:02X?}", &data[pos..(pos + 8)]);
        pos += 8;
        for _n in 0..4 {
            println!("Received data: {:02X?}", &data[pos..(pos + 30)]);
            dumpblock_parse_one(data[pos..(pos + 30)].to_vec());
            pos += 30;
        }
    }
}

fn dumpblock_parse(data: Vec<u8>) {
    // TODO print offset for verbosity
    let mut pos = 0;
    while pos < data.len() {
        pos += 8;
        for _n in 0..4 {
            dumpblock_parse_one(data[pos..(pos + 30)].to_vec());
            pos += 30;
        }
    }
}

fn dumpblock_parse_one(value: Vec<u8>) {
    if value[0] == 0xff {
        println!("  (empty data)");
        return;
    }
    if value[0] == 0x50 {
        println!("  (no coordinates)");
        return;
    }
    if value[0] == 0x41 {
        println!("  (? new track, no data)");
        return;
    }
    if value[0] == 0x43 {
        println!(
            "  (? button pressed, note in next waypoint, this record doesn't contain coordinates)"
        );
        return;
    }
    if value[0] == 0x42 {
        println!("  (switch-off. not to gpx)");
        return; // we could dump this
    }

    let sat_used = value[1] & 0x0f;
    let sat_visib = (value[1] & 0xf0) >> 4;
    let ymd = u32::from_be_bytes(value[2..6].try_into().unwrap());
    let secs = u16::from_le_bytes(value[6..8].try_into().unwrap());
    let mins = (ymd & 0x3f) as u8;
    let hour = (ymd >> 6 & 0x1f) as u8;
    let day = (ymd >> 11 & 0x1f) as u8;
    let mon = (ymd >> 16 & 0xf) as u8;
    let year = value[2] as u16 + 2000;

    // ymd_opt is deprecated, but the recommended with_ymd_and_hms doesn't suppport millis
    let time = Utc
        .ymd_opt(year as i32, mon as u32, day as u32)
        .unwrap()
        .and_hms_milli_opt(
            hour as u32,
            mins as u32,
            secs as u32 / 1000,
            (secs % 1000) as u32,
        )
        .unwrap();

    let course = u16::from_le_bytes(value[28..30].try_into().unwrap()) as f32 / 100.0;
    let speed = u16::from_le_bytes(value[26..28].try_into().unwrap()) as f32 / 100.0;
    let hdop = value[8] as f32 / 10.0;
    let ele = u16::from_le_bytes(value[22..24].try_into().unwrap()) as f32 / 100.0;
    let lat = u32::from_le_bytes(value[14..18].try_into().unwrap()) as f32 / 10000000.0;
    let lon = u32::from_le_bytes(value[18..22].try_into().unwrap()) as f32 / 10000000.0;

    println!(
        "
      <trkpt lat=\"{lat}\" lon=\"{lon}\">
        <ele>{ele}</ele>
        <time>{time}</time>
        <sat>{sat_used}</sat>
        <hdop>{hdop}</hdop>
        <extensions>
          <gpxtpx:TrackPointExtension>
            <gpxtpx:speed>{speed}</gpxtpx:speed>
            <gpxtpx:course>{course}</gpxtpx:course>
          </gpxtpx:TrackPointExtension>
          <mat:TrackPointExtension>
            <mat:sat_view>{sat_visib}</mat:sat_view>
          </mat:TrackPointExtension>
        </extensions>
      </trkpt>
"
    );
}
