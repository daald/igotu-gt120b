mod comm_bulk;
mod commands;
mod gt120b_datadump;
mod gt120b_workflow;
mod intf;
mod intf_bulk;
mod intf_file;
use crate::comm_bulk::CommBulk;
use crate::gt120b_workflow::workflow;
use crate::intf::Intf;
use crate::intf_bulk::IntfBulk;
use crate::intf_file::IntfFile;
use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Use a real device instead of a simulation
    #[arg(short, long, default_value_t = false)]
    real: bool,

    /// Number of times to greet
    //#[arg(short, long, default_value_t = 1)]
    //count: u8,

    /// Run some more commands to match replay file
    #[arg(short, long, default_value_t = false)]
    bestreplay: bool,

    #[arg(long, default_value_t = true)]
    orig_sw_equivalent: bool,

    //let replay_file = "src/replay-120b.txt";
    //let replay_file = "src/gt-120b-kvm-sesson-20250529.json.txt";
    //let replay_file = "src/gt-120b-kvm-sesson-20250603.json.txt";
    /// Filename of simulation replay file
    #[arg(short, long)]
    sim_file_name: String,
}

fn main() {
    let args = Args::parse();

    env_logger::init();

    //dbg!(&args);

    let intf: Box<dyn Intf> = if args.real {
        Box::new(IntfBulk::new())
    } else {
        Box::new(IntfFile::new(args.sim_file_name))
    };
    let mut comm = CommBulk { intf: intf };

    workflow(&mut comm, args.bestreplay, args.orig_sw_equivalent);

    println!("END");
}
