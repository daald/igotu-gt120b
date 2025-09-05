use chrono::{DateTime, Local, NaiveDate, TimeZone, Utc};
use log::trace;
use std::fs::File;
use std::io::{BufWriter, Result, Write};

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
    pub fn dump(&self, f: &mut BufWriter<File>) -> Result<()> {
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
                writeln!(
                    f,
                    "      <trkpt lat=\"{lat}\" lon=\"{lon}\">
        <ele>{ele}</ele>
        <time>{}</time>{}
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
                    time.to_rfc3339(),
                    if *wpflags != 0 {
                        format!("\n        <type>WpFlag:{wpflags}</type>")
                    } else {
                        "".to_string()
                    }
                )?;
            }
            _ => (),
        }
        Ok(())
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
        self.parse_data(data);
        //       dumpblock_parse(data);
    }
    pub fn write_out(&mut self, conf_change_every_day: bool, meta_desc: &String) -> Result<usize> {
        self.waypoints.sort_by(|a, b| a.time().cmp(&b.time()));

        fn start_file(name: &str, meta_desc: &String) -> Result<Option<BufWriter<File>>> {
            println!("Writing gpx file {name}");
            let f = File::create(name)?;
            let mut fbuf = BufWriter::new(f);
            assert!(fbuf.capacity() > 0);
            writeln!(&mut fbuf,"﻿<?xml version=\"1.0\" encoding=\"utf-8\" standalone=\"no\"?>
<!-- generated using test of rust implementation -->
<gpx version=\"1.1\" creator=\"igotU_GPS_WIN\" xmlns:gpxx=\"http://www.garmin.com/xmlschemas/GpxExtensions/v3\" xmlns:gpxwpx=\"http://www.garmin.com/xmlschemas/WaypointExtension/v1\" xmlns:gpxtpx=\"http://www.garmin.com/xmlschemas/TrackPointExtension/v2\" xmlns:mat=\"http://www.mobileaction.com/xmlschemas/TrackPointExtension/v2\" xmlns=\"http://www.topografix.com/GPX/1/1\">
  <metadata>
    <desc>{meta_desc}</desc>
  </metadata>
  <trk>
    <trkseg>")?;
            fbuf.flush()?;
            Ok(Some(fbuf))
        }
        fn end_file(ref mut f_ref: Option<BufWriter<File>>) -> Result<()> {
            if let Some(f) = f_ref {
                writeln!(
                    f,
                    "    </trkseg>
  </trk>
</gpx>"
                )?;
                f.flush()?;
            }
            Ok(())
        }

        let mut lastday = NaiveDate::MIN;
        fn set_daychange(time: &DateTime<Utc>, lastday: &mut NaiveDate) {
            let localdatetime: DateTime<Local> = DateTime::from(*time);
            let day = localdatetime.date_naive();
            *lastday = day;
        }
        fn need_daychange(time: &DateTime<Utc>, lastday: &mut NaiveDate) -> bool {
            let localdatetime: DateTime<Local> = DateTime::from(*time);
            let day = localdatetime.date_naive();
            if day != *lastday {
                set_daychange(time, lastday);
                return true;
            } else {
                return false;
            }
        }

        self.transfer_flags_reverse();
        self.transfer_flags_forward();

        let mut f_ref: Option<BufWriter<File>> = None;
        let mut filenum = 0;
        for wp in &self.waypoints {
            if let DatablockEnum::Datablock {
                wpflags: _,
                time,
                sat_used: _,
                sat_visib: _,
                course: _,
                speed: _,
                hdop: _,
                ele: _,
                lat: _,
                lon: _,
            } = wp
            {
                if f_ref.is_some() {
                    if conf_change_every_day && need_daychange(time, &mut lastday) {
                        end_file(f_ref)?;
                        f_ref = None;
                    }
                }
                if f_ref.is_none() {
                    filenum += 1;
                    f_ref = start_file(
                        &format!("testout-{:02}.gpx", filenum).to_string(),
                        meta_desc,
                    )?;
                    set_daychange(time, &mut lastday);
                }
                wp.dump(f_ref.as_mut().expect("at this stage, file is always open"))?;
                if f_ref.is_some() {
                    if wp.is_eof() {
                        end_file(f_ref)?;
                        f_ref = None;
                    }
                }
            }
        }
        if f_ref.is_some() {
            end_file(f_ref)?;
        }
        println!("Exported {filenum} files");
        Ok(filenum)
    }

    fn transfer_flags_forward(&mut self) {
        let mut next_flags = 0u8;
        for wp in self.waypoints.iter_mut() {
            match wp {
                DatablockEnum::NoBlock => {}
                DatablockEnum::PrevMod(_, _) => {
                    next_flags = 0;
                }
                DatablockEnum::NextMod(_, wpflags) => {
                    next_flags |= *wpflags;
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
    }
    fn transfer_flags_reverse(&mut self) {
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
    }

    fn parse_data(&mut self, data: Vec<u8>) {
        // TODO print offset for verbosity
        let mut pos = 0;
        while pos < data.len() {
            trace!("< {:02X?}", &data[pos..(pos + 8)]);
            pos += 8;
            for _n in 0..4 {
                trace!("< {:02X?}", &data[pos..(pos + 30)]);
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
        // empty data
        return DatablockEnum::NoBlock;
    }
    if value[0] == 0x50 {
        // block without coordinates
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
        .ymd(year as i32, mon as u32, day as u32)
        .and_hms_milli_opt(
            hour as u32,
            mins as u32,
            (secs / 1000) as u32,
            (secs % 1000) as u32,
        )
        .unwrap();

    if value[0] == 0x41 {
        // new track, no geo
        return DatablockEnum::NextMod(time, 0x01);
    }
    if value[0] == 0x42 {
        // switch-off. geo but no waypoint
        return DatablockEnum::PrevMod(time, 0x02); // we could dump this. but orig sw ignores this coords and only takes the flag
    }
    if value[0] == 0x43 {
        // button pressed, no geo
        return DatablockEnum::NextMod(time, 0x10);
    }

    let sat_used = value[1] & 0x0f;
    let sat_visib = (value[1] & 0xf0) >> 4;
    let course = u16::from_le_bytes(value[28..30].try_into().unwrap()) as f32 / 100.0;
    let speed = u16::from_le_bytes(value[26..28].try_into().unwrap()) as f32 / 100.0;
    let hdop = value[8] as f32 / 10.0;
    let ele = i32::from_le_bytes(value[22..26].try_into().unwrap()) as f32 / 100.0;
    let lat = i32::from_le_bytes(value[14..18].try_into().unwrap()) as f32 / 10000000.0;
    let lon = i32::from_le_bytes(value[18..22].try_into().unwrap()) as f32 / 10000000.0;

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
