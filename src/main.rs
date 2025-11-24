use env_logger::Builder;
use env_logger::Env;
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

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Clear device memory after successfully downloading and writing gpx files
    #[arg(short, long, default_value_t = false)]
    clear: bool,

    /// Run some extra commands without known purpose to match replay file
    #[arg(long, default_value_t = false)]
    orig_sw_workflow: bool,

    /// Use the exact same meta format as the original software. By default, the format is more verbose and not base64 encoded
    #[arg(long, default_value_t = true)]
    orig_sw_meta: bool,

    /// Simulate using specified replay file instead of real hardware access
    #[arg(long)]
    sim_file_name: Option<String>,

    /// filename part on the left side of the date, including optional path
    #[arg(short, long, default_value = "")]
    prefix: String,

    /// filename part on the right side of the date
    #[arg(short, long, default_value = "")]
    suffix: String,
}

fn main() {
    let args = Args::parse();

    let env = Env::new().filter_or("RUST_LOG", "info");
    Builder::from_env(env).init();

    //dbg!(&args);

    let intf: Box<dyn Intf> = if args.sim_file_name.is_none() {
        Box::new(IntfBulk::new())
    } else {
        Box::new(IntfFile::new(args.sim_file_name.unwrap()))
    };
    let mut comm = CommBulk::new(intf);

    workflow(
        &mut comm,
        args.clear,
        args.orig_sw_workflow,
        args.orig_sw_meta,
        args.prefix,
        args.suffix,
    );

    println!("Completed.");
}
