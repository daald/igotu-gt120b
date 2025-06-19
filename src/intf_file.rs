use std::fs::read_to_string;
//use futures_lite::future::block_on;
//use hex_literal::hex;    //use: hex!
use hex;


use crate::intf;
pub use intf::Intf;


pub struct IntfFile {
  lines: Vec<InOut>,
  nextLine: usize,
}

struct InOut {
  out: bool,
  line: Vec<u8>,
}

pub fn init_intf_file() -> IntfFile {
    println!("\n\nRUNNING SIMULATOR\n\n");

    let mut result = Vec::new();

    for line in read_to_string("src/replay-120b.txt").unwrap().lines() {
        if line.starts_with("> "){
            result.push(InOut{out: true, line: hex::decode(line.to_string()[2..].replace(":", "")).expect("Decoding failed")});
        } else if line.starts_with("< "){
            result.push(InOut{out: false, line: hex::decode(line.to_string()[2..].replace(":", "")).expect("Decoding failed")});
        } else {
            println!("Unknown line {line}");
        }
    }
    return IntfFile{lines: result, nextLine: 0};
}

impl Intf for IntfFile {
  fn send_and_receive(&mut self, to_device: Vec<u8>) -> Vec<u8> {
    let outLine = &self.lines[self.nextLine];
    if !outLine.out {
      panic!("No out-line: #{}", self.nextLine);
    }
    if outLine.line != to_device {
      panic!("Next output doesn't match: #{}\nactual: {:02X?}\nexpected: {:02X?}", self.nextLine, outLine.line, to_device);
    }
    
    self.nextLine += 1;
    let inLine = &self.lines[self.nextLine];
    if inLine.out {
      panic!("No in-line: #{}", self.nextLine);
    }
    self.nextLine += 1;
    return inLine.line.clone();
  }

  fn is_real(&self) -> bool {
    return false;
  }
}

