use chrono::{DateTime, Local, NaiveDate, Utc};
use log::{info, trace};
use std::fs::File;
use std::io::{BufWriter, Result, Write};

#[derive(Debug)]
struct Waypoint {
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
}

#[derive(Debug)]
enum DatablockEnum {
    Datablock(Waypoint),
    PrevMod(DateTime<Utc>, u8),
    NextMod(DateTime<Utc>, u8),
    NoBlock,
}

impl DatablockEnum {
    pub fn dump<T: std::io::Write>(&self, f: &mut BufWriter<T>) -> Result<()> {
        match self {
            DatablockEnum::Datablock(wpt) => {
                writeln!(
                    f,
                    "      <trkpt lat=\"{}\" lon=\"{}\">
        <ele>{}</ele>
        <time>{}</time>{}
        <sat>{}</sat>
        <hdop>{}</hdop>
        <extensions>
          <gpxtpx:TrackPointExtension>
            <gpxtpx:speed>{}</gpxtpx:speed>
            <gpxtpx:course>{}</gpxtpx:course>
          </gpxtpx:TrackPointExtension>
          <mat:TrackPointExtension>
            <mat:sat_view>{}</mat:sat_view>
          </mat:TrackPointExtension>
        </extensions>
      </trkpt>",
                    &wpt.lat,
                    &wpt.lon,
                    &wpt.ele,
                    &wpt.time.to_rfc3339(),
                    if wpt.wpflags != 0 {
                        format!("\n        <type>WpFlag:{}</type>", &wpt.wpflags)
                    } else {
                        "".to_string()
                    },
                    &wpt.sat_used,
                    &wpt.hdop,
                    &wpt.speed,
                    &wpt.course,
                    &wpt.sat_visib,
                )?;
            }
            _ => (),
        }
        Ok(())
    }

    pub fn is_eof(&self) -> bool {
        match self {
            DatablockEnum::Datablock(wp) => (wp.wpflags & 0x02) != 0,
            DatablockEnum::PrevMod(_, flags) => (*flags & 0x02) != 0,
            _ => false,
        }
    }
    pub fn time(&self) -> DateTime<Utc> {
        match self {
            DatablockEnum::Datablock(wp) => wp.time,
            DatablockEnum::PrevMod(time, _) => *time,
            DatablockEnum::NextMod(time, _) => *time,
            DatablockEnum::NoBlock => DateTime::UNIX_EPOCH,
        }
    }
}

pub struct Gt120bDataDump {
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
    }
    pub fn write_out(&mut self, conf_change_every_day: bool, meta_desc: &String) -> Result<usize> {
        self.waypoints.sort_by(|a, b| a.time().cmp(&b.time()));

        fn start_file(name: &str, meta_desc: &String) -> Result<Option<BufWriter<File>>> {
            info!("Writing gpx file {name}");
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
            if let DatablockEnum::Datablock(wpt) = wp {
                if f_ref.is_some() {
                    if conf_change_every_day && need_daychange(&wpt.time, &mut lastday) {
                        end_file(f_ref)?;
                        f_ref = None;
                    }
                }
                if f_ref.is_none() {
                    filenum += 1;
                    f_ref = start_file(
                        &format!("testout-{}.gpx", wpt.time.format("%Y-%m-%d_%H-%M")).to_string(),
                        meta_desc,
                    )?;
                    set_daychange(&wpt.time, &mut lastday);
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
        info!("Exported {filenum} files");
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
                DatablockEnum::Datablock(wpt) => {
                    wpt.wpflags |= next_flags;
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
                DatablockEnum::Datablock(wpt) => {
                    wpt.wpflags |= next_flags;
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
    let fullmsecs = u16::from_le_bytes(value[6..8].try_into().unwrap()) as u32;
    let secs = fullmsecs / 1000;
    let msecs = fullmsecs % 1000;
    let mins = ymd & 0x3f;
    let hour = ymd >> 6 & 0x1f;
    let day = ymd >> 11 & 0x1f;
    let mon = ymd >> 16 & 0xf;
    let year = 2000 + value[2] as i32;

    let time = NaiveDate::from_ymd_opt(year, mon, day)
        .unwrap()
        .and_hms_milli_opt(hour, mins, secs, msecs)
        .unwrap()
        .and_utc();

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

    DatablockEnum::Datablock(Waypoint {
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn parse_datablock_NoBlock_goodcase() {
        let input=hex!["ff ff ff ff ff ff ff ff ff ff ff ff ff ff ff ff ff ff ff ff ff ff ff ff ff ff ff ff ff ff"].to_vec();

        let result = parse_datablock(input);

        println!("{:?}", result);
        assert!(matches!(result, DatablockEnum::NoBlock));
    }

    #[test]
    fn parse_datablock_NextMod_switchOn() {
        let input=hex!["41 a0 19 07 fd 05 f4 15 00 00 0c 00 00 19 00 00 00 00 00 00 00 00 3a c0 00 00 00 00 00 00"].to_vec();

        let result = parse_datablock(input);

        println!("{:?}", result);
        assert!(matches!(result, DatablockEnum::NextMod(_, _)));
        let DatablockEnum::NextMod(time, flags) = result else {
            panic!("Invalid result type")
        };
        //        assert_eq!(wpt.time, 0);
        assert_eq!(flags, 0x01);
    }

    #[test]
    fn parse_datablock_PrevMod_switchOff() {
        let input=hex!["42 a6 19 07 fd 08 69 b5 2a 00 00 00 00 00 2c 95 3b 1c ce 55 18 05 b6 a3 00 00 19 00 c0 0f"].to_vec();

        let result = parse_datablock(input);

        println!("{:?}", result);
        assert!(matches!(result, DatablockEnum::PrevMod(_, _)));
        let DatablockEnum::PrevMod(time, flags) = result else {
            panic!("Invalid result type")
        };
        //        assert_eq!(wpt.time, 0);
        assert_eq!(flags, 0x02);
    }

    #[test]
    fn parse_datablock_Datablock_goodcase() {
        let input=hex!["00 a4 19 07 fd 08 99 ad 2a 00 2b 35 00 19 2c 95 3b 1c cc 61 18 05 c2 ab 00 00 73 00 cd 1e"].to_vec();

        let result = parse_datablock(input);

        println!("{:?}", result);
        assert!(matches!(result, DatablockEnum::Datablock(_)));
        let DatablockEnum::Datablock(wpt) = result else {
            panic!("Invalid result type")
        };
        //        assert_eq!(wpt.time, 0);
        assert_eq!(wpt.wpflags, 0x00);
        assert_eq!(wpt.sat_used, 4);
        assert_eq!(wpt.sat_visib, 10);
        assert_eq!(wpt.course, 78.85);
        assert_eq!(wpt.speed, 1.15);
        assert_eq!(wpt.hdop, 4.2);
        assert_eq!(wpt.ele, 439.7);
        assert_eq!(wpt.lat, 47.366684);
        assert_eq!(wpt.lon, 8.548398);
    }

    #[test]
    fn parse_datablock_NextMod_button() {
        let input=hex!["43 a0 19 07 fd 07 71 e0 00 00 0c 00 00 19 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00"].to_vec();

        let result = parse_datablock(input);

        println!("{:?}", result);
        assert!(matches!(result, DatablockEnum::NextMod(_, _)));
        let DatablockEnum::NextMod(time, flags) = result else {
            panic!("Invalid result type")
        };
        //        assert_eq!(wpt.time, 0);
        assert_eq!(flags, 0x10);
    }

    #[test]
    fn dump() {
        let input = Waypoint {
            time: NaiveDate::from_ymd_opt(2025, 7, 31)
                .unwrap()
                .and_hms_milli_opt(20, 8, 44, 441)
                .unwrap()
                .and_utc(),
            wpflags: 18,
            sat_used: 4,
            sat_visib: 10,
            course: 78.85,
            speed: 1.15,
            hdop: 4.2,
            ele: 439.7,
            lat: 47.366684,
            lon: 8.548398,
        };
        let mut buf = Vec::<u8>::new();
        let mut writer = BufWriter::new(buf);

        DatablockEnum::Datablock(input).dump(&mut writer).unwrap();

        let s = String::from_utf8(writer.into_inner().unwrap()).unwrap();
        assert_eq!(
            s,
            "      \
      <trkpt lat=\"47.366684\" lon=\"8.548398\">
        <ele>439.7</ele>
        <time>2025-07-31T20:08:44.441+00:00</time>
        <type>WpFlag:18</type>
        <sat>4</sat>
        <hdop>4.2</hdop>
        <extensions>
          <gpxtpx:TrackPointExtension>
            <gpxtpx:speed>1.15</gpxtpx:speed>
            <gpxtpx:course>78.85</gpxtpx:course>
          </gpxtpx:TrackPointExtension>
          <mat:TrackPointExtension>
            <mat:sat_view>10</mat:sat_view>
          </mat:TrackPointExtension>
        </extensions>
      </trkpt>
"
        );
    }
}
