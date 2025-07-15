// $ cargo build && rustfmt ./doc/topics/dataformat-parsing/data.rs && ./target/debug/data

use hex_literal::hex; //use: hex!

/*
QDateTime IgotuPoint::dateTime() const
{
    const unsigned date = qFromBigEndian<quint32>
        (reinterpret_cast<const uchar*>(record.data())) & 0x00ffffff;
    const unsigned sec = qFromBigEndian<quint16>
        (reinterpret_cast<const uchar*>(record.data()) + 4);

    return QDateTime(
        QDate(
          ((QDate::currentDate().year() + 4 - ((date >> 20) & 0xf)) & 0xfff0) + ((date >> 20) & 0xf),
          (date >> 16) & 0xf,
          (date >> 11) & 0x1f),
        QTime((date >> 6) & 0x1f, date & 0x3f, sec / 1000, sec % 1000),
        Qt::UTC);
}
*/

fn t(value: Vec<u8>) {
    println!();
    println!("{value:02x?}");
    let date = u32::from_be_bytes(value[0..4].try_into().unwrap()); //& 0x00ffffff;
    let date3a = u16::from_le_bytes(value[6..8].try_into().unwrap());
    let date3b = u32::from_le_bytes(value[4..8].try_into().unwrap()); // best choice
    let date2 = u64::from_le_bytes(value.try_into().unwrap());
    dbg!(&date);

    /*
    {
            let y = ((date >> 20) & 0xf);
            let year = ((2025 + 4 - ((date >> 20) & 0xf)) & 0xfff0) + ((date >> 20) & 0xf);
            let m = (date >> 16) & 0xf;
            let d = (date >> 11) & 0x1f;
            println!("{y} {year} {m} {d}");
        }

        {
            let y = (date >> 20) & 0x1f;
            let m = (date >> 23) & 0xf;
            let d = (date >> 27) & 0x1f;
            println!("{y} {m} {d}");
        }
    */
    {
        println!("{date2:x} {date3a:x} {date3a} {date3b:x} {date3b}");
        //        let d = (date2) & 0x1f;
        println!(
            "  {} {} {}",
            (date >> 16) & 0xf,
            (date >> 11) & 0x1f,
            (date) & 0x1f
        );
    }
    {
        let mut value = date3b;
        for n in 0..32 {
            print!("  {:-2},{:-2}", value & 0xf, value & 0x1f,);
            value >>= 1;
        }
        println!();
    }
}

fn main() {
    // example from 2025-07-31
    t(hex!("0c:00:41:02:98:44:03:cc").to_vec());
    t(hex!("0c:00:41:02:0f:e9:2a:cc").to_vec());
    t(hex!("0c:00:41:02:6e:31:e3:e0").to_vec());
    t(hex!("0c:00:41:02:5a:1c:39:e1").to_vec());
}
