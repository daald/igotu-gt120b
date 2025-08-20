use chrono::{DateTime, TimeZone, Utc};

enum DatablockEnum {
    Datablock {
        time: DateTime<Utc>,
        wpflags: u8,
        sat_used: u8,
        sat_visib: u8,
        course: f32,
        speed: f32,
        hdop: f32,
        ele: f32,
        lat: f32,
        lon: f32,
    },
    PrevMod(DateTime<Utc>, u8),
    NextMod(DateTime<Utc>, u8),
    NoBlock,
}

impl DatablockEnum {
    pub fn dump(&self) {
        match self {
            DatablockEnum::Datablock {
                time,
                wpflags,
                sat_used,
                sat_visib,
                course,
                speed,
                hdop,
                ele,
                lat,
                lon,
            } => {
                println!(
                    "      <trkpt lat=\"{lat}\" lon=\"{lon}\">
        <ele>{ele}</ele>
        <time>{time}</time>{}
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
      </trkpt>",
                    if *wpflags != 0 {
                        format!("\n        <type>WpFlag:{wpflags}</type>")
                    } else {
                        "".to_string()
                    }
                );
            }
            _ => (),
        }
    }

    pub fn is_eof(&self) -> bool {
        match self {
            DatablockEnum::Datablock {
                wpflags,
                time: _,
                sat_used: _,
                sat_visib: _,
                course: _,
                speed: _,
                hdop: _,
                ele: _,
                lat: _,
                lon: _,
            } => (*wpflags & 0x02) != 0,
            DatablockEnum::PrevMod(_, flags) => (*flags & 0x02) != 0,
            _ => false,
        }
    }
    pub fn is_datapoint(&self) -> bool {
        match self {
            DatablockEnum::Datablock {
                time: _,
                wpflags: _,
                sat_used: _,
                sat_visib: _,
                course: _,
                speed: _,
                hdop: _,
                ele: _,
                lat: _,
                lon: _,
            } => true,
            _ => false,
        }
    }
    pub fn time(&self) -> DateTime<Utc> {
        match self {
            DatablockEnum::Datablock {
                time,
                wpflags: _,
                sat_used: _,
                sat_visib: _,
                course: _,
                speed: _,
                hdop: _,
                ele: _,
                lat: _,
                lon: _,
            } => *time,
            DatablockEnum::PrevMod(time, _) => *time,
            DatablockEnum::NextMod(time, _) => *time,
            DatablockEnum::NoBlock => DateTime::UNIX_EPOCH,
        }
    }
}

pub struct Gt120bDataDump {
    //device: Device,
    //interface: Interface,
    waypoints: Vec<DatablockEnum>,
}

impl Gt120bDataDump {
    pub fn new() -> Self {
        Gt120bDataDump {
            waypoints: Vec::new(),
        }
    }

    pub fn process_datablock(&mut self, data: Vec<u8>) {
        let structsize = 8 + 4 * 30;
        assert_eq!(0, data.len() % structsize);
        self.dumpblock_hex(data);
        //       dumpblock_parse(data);
    }

    pub fn close() {
        //TODO apply wpflags
        //TODO print out everything
    }
    pub fn write_out(&mut self) {
        self.waypoints.sort_by(|a, b| a.time().cmp(&b.time()));
        let mut next_flags = 0u8;
        for wp in self.waypoints.iter_mut().rev() {
            match wp {
                DatablockEnum::NoBlock => {}
                DatablockEnum::PrevMod(_, wpflags) => {
                    next_flags |= *wpflags;
                }
                DatablockEnum::NextMod(_, _) => {
                    next_flags = 0;
                }
                DatablockEnum::Datablock {
                    wpflags,
                    time: _,
                    sat_used: _,
                    sat_visib: _,
                    course: _,
                    speed: _,
                    hdop: _,
                    ele: _,
                    lat: _,
                    lon: _,
                } => {
                    *wpflags |= next_flags;
                    next_flags = 0;
                }
            }
        }
        let mut next_flags = 0u8;
        for wp in self.waypoints.iter_mut() {
            match wp {
                DatablockEnum::NoBlock => {}
                DatablockEnum::NextMod(_, wpflags) => {
                    next_flags |= *wpflags;
                }
                DatablockEnum::PrevMod(_, _) => {
                    next_flags = 0;
                }
                DatablockEnum::Datablock {
                    wpflags,
                    time: _,
                    sat_used: _,
                    sat_visib: _,
                    course: _,
                    speed: _,
                    hdop: _,
                    ele: _,
                    lat: _,
                    lon: _,
                } => {
                    *wpflags |= next_flags;
                    next_flags = 0;
                }
            }
        }
        let mut started = false;
        for wp in &self.waypoints {
            if !wp.is_datapoint() {
                continue;
            }
            if !started {
                println!("﻿<?xml version=\"1.0\" encoding=\"utf-8\" standalone=\"no\"?>
<gpx version=\"1.1\" creator=\"igotU_GPS_WIN\" xmlns:gpxx=\"http://www.garmin.com/xmlschemas/GpxExtensions/v3\" xmlns:gpxwpx=\"http://www.garmin.com/xmlschemas/WaypointExtension/v1\" xmlns:gpxtpx=\"http://www.garmin.com/xmlschemas/TrackPointExtension/v2\" xmlns:mat=\"http://www.mobileaction.com/xmlschemas/TrackPointExtension/v2\" xmlns=\"http://www.topografix.com/GPX/1/1\">
  <metadata>
    <desc>//TODO</desc>
  </metadata>
  <trk>
    <trkseg>");
                started = true;
            }
            wp.dump();
            if started && wp.is_eof() {
                println!(
                    "    </trkseg>
  </trk>
</gpx>"
                );
                started = false;
            }
        }
    }

    fn dumpblock_hex(&mut self, data: Vec<u8>) {
        // TODO print offset for verbosity
        let mut pos = 0;
        while pos < data.len() {
            println!("Received data: {:02X?}", &data[pos..(pos + 8)]);
            pos += 8;
            for _n in 0..4 {
                println!("Received data: {:02X?}", &data[pos..(pos + 30)]);
                let wp = parse_datablock(data[pos..(pos + 30)].to_vec());
                if !matches!(wp, DatablockEnum::NoBlock) {
                    self.waypoints.push(wp);
                }
                pos += 30;
            }
        }
    }
}
fn dumpblock_parse(data: Vec<u8>) {
    // TODO print offset for verbosity
    let mut pos = 0;
    while pos < data.len() {
        pos += 8;
        for _n in 0..4 {
            parse_datablock(data[pos..(pos + 30)].to_vec());
            pos += 30;
        }
    }
}

fn parse_datablock(value: Vec<u8>) -> DatablockEnum {
    if value[0] == 0xff {
        println!("  (empty data)");
        return DatablockEnum::NoBlock;
    }
    if value[0] == 0x50 {
        println!("  (no coordinates)");
        return DatablockEnum::NoBlock;
    }

    let ymd = u32::from_be_bytes(value[2..6].try_into().unwrap());
    let secs = u16::from_le_bytes(value[6..8].try_into().unwrap());
    let mins = (ymd & 0x3f) as u8;
    let hour = (ymd >> 6 & 0x1f) as u8;
    let day = (ymd >> 11 & 0x1f) as u8;
    let mon = (ymd >> 16 & 0xf) as u8;
    let year = 2000 + value[2] as u16;

    // ymd_opt is deprecated, but the recommended with_ymd_and_hms doesn't suppport millis
    let time = Utc
        .ymd_opt(year as i32, mon as u32, day as u32)
        .unwrap()
        .and_hms_milli_opt(
            hour as u32,
            mins as u32,
            (secs / 1000) as u32,
            (secs % 1000) as u32,
        )
        .unwrap();

    if value[0] == 0x41 {
        println!("  (? new track, no data) WpFlags of following + 0x01");
        return DatablockEnum::NextMod(time, 0x01);
    }
    if value[0] == 0x42 {
        println!("  (switch-off. not to gpx) WpFlags of previous + 0x02");
        return DatablockEnum::PrevMod(time, 0x02); // we could dump this. but orig sw ignores this coords and only takes the flag
    }
    if value[0] == 0x43 {
        println!(
            "  (? button pressed, note in next waypoint, this record doesn't contain coordinates) WpFlags of following + 0x10"
        );
        return DatablockEnum::NextMod(time, 0x10);
    }

    let sat_used = value[1] & 0x0f;
    let sat_visib = (value[1] & 0xf0) >> 4;
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
    DatablockEnum::Datablock {
        time: time,
        wpflags: 0,
        sat_used: sat_used,
        sat_visib: sat_visib,
        course: course,
        speed: speed,
        hdop: hdop,
        ele: ele,
        lat: lat,
        lon: lon,
    }
}
