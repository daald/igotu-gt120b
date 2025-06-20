use std::fs::read_to_string;
//use futures_lite::future::block_on;
//use hex_literal::hex;    //use: hex!
use hex;

use crate::intf;
pub use intf::Intf;

pub struct IntfFile {
    lines: Vec<InOut>,
    next_line: usize,
}

struct InOut {
    out: bool,
    line: Vec<u8>,
    line_num: usize,
}

impl IntfFile {
    pub fn new() -> Self {
        println!("\n\nRUNNING SIMULATOR\n\n");

        let mut result = Vec::new();

        let mut line_num = 0;
        for line in read_to_string("src/replay-120b.txt").unwrap().lines() {
            line_num += 1;
            if line.starts_with("#") {
            } else if line.starts_with("> ") {
                result.push(InOut {
                    out: true,
                    line: hex::decode(line.to_string()[2..].replace(":", ""))
                        .expect("Decoding failed"),
                    line_num: line_num,
                });
            } else if line.starts_with("< ") {
                result.push(InOut {
                    out: false,
                    line: hex::decode(line.to_string()[2..].replace(":", ""))
                        .expect("Decoding failed"),
                    line_num: line_num,
                });
            } else {
                println!("Unknown line {line}");
            }
        }
        return Self {
            lines: result,
            next_line: 0,
        };
    }
}

impl Intf for IntfFile {
    fn send_and_receive(&mut self, to_device: Vec<u8>) -> Vec<u8> {
        let out_line = &self.lines[self.next_line];
        if !out_line.out {
            panic!("No cmd-line: #{}", out_line.line_num);
        }
        if out_line.line != to_device {
            panic!(
                "Next cmd doesn't match: #{}\nactual: {:02X?}\nexpected: {:02X?}",
                out_line.line_num, out_line.line, to_device
            );
        }

        self.next_line += 1;
        let in_line = &self.lines[self.next_line];
        if in_line.out {
            panic!("No response line: #{}", in_line.line_num);
        }
        self.next_line += 1;
        return in_line.line.clone();
    }

    fn is_real(&self) -> bool {
        return false;
    }
}
