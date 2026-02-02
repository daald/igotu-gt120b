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
use clap::Subcommand;

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
    #[arg(long, default_value_t = false)]
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

    #[command(subcommand)]
    command: Option<SubCommands>,
}

#[derive(Subcommand, Debug, PartialEq)]
enum SubCommands {
    /// Show an example udev rules file
    Rules,
}

fn main() {
    let args = Args::parse();

    if Some(SubCommands::Rules) == args.command {
        println!("
# /etc/udev/rules.d/51-igotu-b-series.rules

# GT-120b
SUBSYSTEMS==\"usb\", ATTRS{{idVendor}}==\"0df7\", ATTRS{{idProduct}}==\"0920\", GROUP=\"plugdev\", MODE=\"0664\"
"
);
        return;
    }

    let env = Env::new().filter_or("RUST_LOG", "info");
    Builder::from_env(env).init();

    //dbg!(&args);

    let intf: Box<dyn Intf> = if let Some(sim_file_name) = args.sim_file_name {
        Box::new(IntfFile::new(sim_file_name))
    } else {
        Box::new(IntfBulk::new())
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
