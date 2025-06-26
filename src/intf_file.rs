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
    comment: String,
}

impl IntfFile {
    pub fn new(file_name: String) -> Self {
        //let replay_file = "src/replay-120b.txt";
        //let replay_file = "src/gt-120b-kvm-sesson-20250529.json.txt";
        //let replay_file = "src/gt-120b-kvm-sesson-20250603.json.txt";

        println!("\n\nRUNNING SIMULATOR with file {}\n\n", file_name);

        let mut result = Vec::new();

        let mut line_num = 0;
        let mut next_comment: String = "".to_string();
        for line in read_to_string(file_name).unwrap().lines() {
            line_num += 1;
            let next_isout;
            if line == "" || line.starts_with("#") {
                if line.starts_with("#: ") {
                    next_comment = line[2..].trim().to_string();
                }
                continue;
            } else if line.starts_with("> ") {
                next_isout = true;
            } else if line.starts_with("< ") {
                next_isout = false;
            } else {
                println!("Unknown line {line}");
                continue;
            }
            result.push(InOut {
                out: next_isout,
                line: hex::decode(line.to_string()[2..].replace(":", "")).expect("Decoding failed"),
                line_num: line_num,
                comment: next_comment,
            });
            next_comment = "".to_string();
        }
        return Self {
            lines: result,
            next_line: 0,
        };
    }

    fn sim_send(&mut self, to_device: Vec<u8>) -> &InOut {
        let out_line = &self.lines[self.next_line];
        self.next_line += 1;
        if !out_line.comment.is_empty() {
            println!("SIMULATOR >#{}: {}", out_line.line_num, out_line.comment);
        } else {
            println!("SIMULATOR >#{}", out_line.line_num);
        }
        if !out_line.out {
            panic!("SIMULATOR >#{}: No cmd-line", out_line.line_num);
        }
        if out_line.line != to_device {
            panic!(
                "SIMULATOR >#{}: Next cmd doesn't match:\nactual:   {:02X?}\nexpected: {:02X?}",
                out_line.line_num, to_device, out_line.line
            );
        }
        return out_line;
    }
}

impl Intf for IntfFile {
    fn send_and_receive(&mut self, to_device: Vec<u8>) -> Vec<u8> {
        self.sim_send(to_device);

        let in_line = &self.lines[self.next_line];
        self.next_line += 1;
        if !in_line.comment.is_empty() {
            println!("SIMULATOR <#{}: {}", in_line.line_num, in_line.comment);
        } else {
            println!("SIMULATOR <#{}", in_line.line_num);
        }
        if in_line.out {
            panic!("SIMULATOR <#{}: Not a response in line", in_line.line_num);
        }
        let mut line = in_line.line.clone();
        while self.next_line < (&self.lines).len() && !&self.lines[self.next_line].out {
            let in_line = &self.lines[self.next_line];
            self.next_line += 1;
            if !in_line.comment.is_empty() {
                println!("SIMULATOR <+#{}: {}", in_line.line_num, in_line.comment);
            }
            line.append(&mut in_line.line.to_vec());
        }
        return line;
    }

    fn cmd_oneway_devicereset(&mut self, to_device: Vec<u8>) {
        let out_line = self.sim_send(to_device);
        println!("SIMULATOR #{}: Device reset now", out_line.line_num);
    }

    fn is_real(&self) -> bool {
        return false;
    }
}
