// answer from chatgpt using 5 input/output combinations from logs: https://chatgpt.com/c/6864f7c2-e034-8010-bbc3-00e200e6264e
fn calculate_out(b: u8, c: u8) -> u32 {
    let out_shifted = ((b as u32) << 3) + ((c as u32) >> 5) + 1;
    out_shifted << 12
}

fn calculate_out_with_bitshift_ops(
    in2_hex: &str,
    in3_hex: &str,
) -> Result<u32, std::num::ParseIntError> {
    // 1. Parse the hex strings into unsigned 32-bit integers.
    let in2 = u8::from_str_radix(in2_hex, 16)?;
    let in3 = u8::from_str_radix(in3_hex, 16)?;

    return Ok(calculate_out(in2, in3));
}

fn testall(b: &str, c: &str, r: u32) {
    let out1 = calculate_out_with_bitshift_ops(b, c).unwrap();
    println!("Inputs: in2=0x{b}, in3=0x{c} -> Output: 0x{:x}", out1); // 0x47000
    assert_eq!(out1, r);
}

fn main() {
    // --- Example 1 ---
    let out1 = calculate_out_with_bitshift_ops("08", "c0").unwrap();
    println!("Inputs: in2=0x08, in3=0xc0 -> Output: 0x{:x}", out1); // 0x47000
    assert_eq!(out1, 0x47000);

    // --- Example 2 ---
    let out2 = calculate_out_with_bitshift_ops("06", "02").unwrap();
    println!("Inputs: in2=0x06, in3=0x02 -> Output: 0x{:x}", out2); // 0x31000
    assert_eq!(out2, 0x31000);

    testall("08", "C0", 0x047000);
    testall("06", "02", 0x031000);

    testall("0B", "8B", 0x05d000);
    testall("08", "C0", 0x047000);
    testall("02", "6E", 0x014000);
    testall("01", "DC", 0x00f000);
    testall("06", "02", 0x031000);
}
