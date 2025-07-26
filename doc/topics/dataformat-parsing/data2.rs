// cargo run --bin data2 && cargo fmt -- ./doc/topics/dataformat-parsing/data2.rs

use chrono::TimeZone;
use chrono::Utc;
use hex_literal::hex; //use: hex!

fn g(_value: Vec<u8>) {
    // unsure what to do with these blocks
}

fn t(value: Vec<u8>) {
    println!();
    println!("{value:02x?}");
    if value[0] == 0xff {
        println!("  (empty data)");
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

#[rustfmt::skip]

fn main() {
    println!("=== from anonymized session 2025-07-28 (dump from 2025-07-31)");

    // #: read (size=0100, pos=001000)
    g(hex!("0c:00:41:02:98:44:03:cc").to_vec());
    t(hex!("41:d0:19:07:fc:46:74:e8:00:00:0c:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00").to_vec());
    t(hex!("43:d4:19:07:fc:47:33:51:00:00:52:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:01:00:5b:1d").to_vec());
    t(hex!("00:d5:19:07:fc:48:00:00:0e:00:7d:3b:00:00:a2:a5:3c:1c:6c:83:15:05:08:c0:00:00:5a:00:49:1d").to_vec()); // <time>2025-07-31T17:08:00Z</time>
    t(hex!("43:d0:19:07:fc:48:7b:32:00:00:0c:00:00:19:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00").to_vec());
    g(hex!("0c:00:41:02:0f:e9:2a:cc").to_vec());
    t(hex!("43:d6:19:07:fc:48:68:42:0e:00:0f:15:00:19:40:a3:3c:1c:88:7b:15:05:da:c0:00:00:15:00:76:23").to_vec());
    t(hex!("00:d8:19:07:fc:48:68:bf:0e:00:11:39:00:19:34:a5:3c:1c:28:81:15:05:18:bf:00:00:0f:00:66:6b").to_vec()); // <time>2025-07-31T17:08:49Z</time>
    t(hex!("00:d9:19:07:fc:49:20:4e:0e:00:11:39:00:19:34:a5:3c:1c:c2:82:15:05:3a:c0:00:00:13:00:a9:63").to_vec()); // <time>2025-07-31T17:09:20Z</time>
    t(hex!("42:d9:19:07:fc:49:20:4e:0e:00:11:39:00:19:34:a5:3c:1c:c2:82:15:05:3a:c0:00:00:13:00:a9:63").to_vec());

    // #: read (size=0f00, pos=001100)
    g(hex!("0c:00:41:02:6e:31:e3:e0").to_vec());
    t(hex!("41:a0:19:07:fd:05:f4:15:00:00:0c:00:00:19:00:00:00:00:00:00:00:00:3a:c0:00:00:00:00:00:00").to_vec());
    t(hex!("43:a6:19:07:fd:07:6e:02:00:00:ac:00:00:19:00:00:00:00:00:00:00:00:3a:c0:00:00:01:00:67:09").to_vec());
    t(hex!("00:a4:19:07:fd:07:b8:88:29:00:cf:35:00:19:b2:96:3b:1c:44:53:18:05:6a:b3:00:00:4e:00:95:24").to_vec()); // <time>2025-07-31T20:07:35Z</time>
    t(hex!("00:a4:19:07:fd:07:10:a4:29:00:0d:42:00:07:bc:96:3b:1c:c4:55:18:05:02:b2:00:00:45:00:58:21").to_vec()); // <time>2025-07-31T20:07:42Z</time>
    g(hex!("0c:00:41:02:5a:1c:39:e1").to_vec());
    t(hex!("43:a0:19:07:fd:07:71:e0:00:00:0c:00:00:19:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00").to_vec());
    t(hex!("00:a4:19:07:fd:08:99:ad:2a:00:2b:35:00:19:2c:95:3b:1c:cc:61:18:05:c2:ab:00:00:73:00:cd:1e").to_vec()); // <time>2025-07-31T20:08:44.441Z</time>
    t(hex!("42:a6:19:07:fd:08:69:b5:2a:00:00:00:00:00:2c:95:3b:1c:ce:55:18:05:b6:a3:00:00:19:00:c0:0f").to_vec());
    t(hex!("ff:ff:ff:ff:ff:ff:ff:ff:ff:ff:ff:ff:ff:ff:ff:ff:ff:ff:ff:ff:ff:ff:ff:ff:ff:ff:ff:ff:ff:ff").to_vec());

    //#         ^  num satellites used <sat>6</sat>
    //#        ^  num satellites in view <mat:sat_view>11</mat:sat_view>
    //#                 -- ^^  increasing number, probably part of time. MSB on the left
    //#                       ^^ ^^  milliseconds including seconds (modulo 1000). LSB
}
