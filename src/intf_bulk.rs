use log::{info, trace};
use nusb::io::EndpointRead;
use nusb::transfer::{Bulk, ControlOut, ControlType, In, Out, Recipient};
use nusb::{Device, DeviceInfo, Interface, MaybeFuture};
use std::io::{Read, Write};
use std::time::Duration;

use std::time::SystemTime;
use std::{thread, time};

use crate::intf;
pub use intf::Intf;

const DEVID_VENDOR: u16 = 0x0df7;
const DEVID_PRODUCT: u16 = 0x0920;
const DEVICE_INTERFACE: u8 = 1;
const BULK_EP_IN: u8 = 0x81;
const BULK_EP_OUT: u8 = 0x01;

pub struct IntfBulk {
    device: Device,
    interface: Interface,
    bus_id: u8,
    device_id: u8,
}

impl Intf for IntfBulk {
    fn send_and_receive(&mut self, to_device: Vec<u8>) -> Vec<u8> {
        let mut reader = self
            .interface
            .endpoint::<Bulk, In>(BULK_EP_IN)
            .unwrap()
            .reader(4096);
        let mut writer = self
            .interface
            .endpoint::<Bulk, Out>(BULK_EP_OUT)
            .unwrap()
            .writer(4096);

        writer.write_all(&to_device).unwrap();
        writer.flush_end().unwrap();

        trace!("  awaiting answer");
        let mut answer = self.read_answer(&mut reader);

        let payloadsize: u16 = u16::from_be_bytes(answer[1..3].try_into().unwrap());
        while answer.len() < payloadsize as usize + 4 {
            trace!("  waiting for more data");
            answer.append(&mut self.read_answer(&mut reader));
        }

        answer
    }

    fn cmd_oneway_devicereset(&mut self, to_device: Vec<u8>) {
        let mut writer = self
            .interface
            .endpoint::<Bulk, Out>(BULK_EP_OUT)
            .unwrap()
            .writer(4096);
        writer.write_all(&to_device).unwrap();
        writer.flush().unwrap();

        info!("Wait for device reset. It will shortly disconnect from USB");
        let (device, device_info, interface) =
            Self::setup_device_and_interface(true, self.bus_id, self.device_id);
        self.device = device;
        self.interface = interface;
        self.bus_id = device_info.busnum();
        self.device_id = device_info.device_address();
    }

    fn get_time_micros(&self) -> u64 {
        let duration_since_epoch = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let timestamp_micros = duration_since_epoch.as_micros();
        timestamp_micros as u64
    }
}

impl IntfBulk {
    pub fn new() -> Self {
        let (device, device_info, interface) = Self::setup_device_and_interface(false, 0xff, 0xff);
        Self {
            device,
            interface,
            bus_id: device_info.busnum(),
            device_id: device_info.device_address(),
        }
    }

    fn wait_for_deviceinfo(wait: bool, bus_id: u8, device_id: u8) -> DeviceInfo {
        let mut sleep_time = 1000;
        loop {
            let di_opt = nusb::list_devices().wait().unwrap().find(|d| {
                d.vendor_id() == DEVID_VENDOR
                    && d.product_id() == DEVID_PRODUCT
                    && match_partially_last_device(d, bus_id, device_id)
            });

            if let Some(di) = di_opt {
                return di;
            }
            if wait {
                thread::sleep(time::Duration::from_millis(sleep_time));
                if sleep_time > 3000 {
                    info!("Still waiting for device to reconnect after reboot");
                    sleep_time = 3000;
                } else if sleep_time < 3000 {
                    sleep_time = sleep_time * 3 / 2;
                }
            } else {
                panic!("Cannot find device");
            }
        }
    }

    fn setup_device_and_interface(
        wait: bool,
        bus_id: u8,
        device_id: u8,
    ) -> (Device, DeviceInfo, Interface) {
        let di = Self::wait_for_deviceinfo(wait, bus_id, device_id);

        info!("USB Device info: {di:?}");

        let mut device = di.open().wait().unwrap();
        let interface = device
            .detach_and_claim_interface(DEVICE_INTERFACE)
            .wait()
            .unwrap();

        Self::ctrl_set_line_state(&mut device);
        (device, di, interface)
    }

    fn read_answer(&mut self, in_queue: &mut EndpointRead<Bulk>) -> Vec<u8> {
        let mut buf = Vec::new();
        in_queue.read_to_end(&mut buf).unwrap();
        buf
    }

    /**
     set control line state request - needed for the device to reply in BULK mode
    */
    fn ctrl_set_line_state(device: &mut Device) {
        println!("Send ctrl_set_line_state");
        device
            .control_out(
                ControlOut {
                    control_type: ControlType::Class,
                    recipient: Recipient::Device,
                    request: 0x22, /* set line state*/
                    value: 0x03,
                    index: 0x00,
                    data: &[],
                },
                Duration::from_secs(3),
            )
            .wait()
            .unwrap();
    }
}

/**
Find a device which is NOT the same deviceid but on the same bus. This is the way the device reconnects to our system
*/
fn match_partially_last_device(devicepath: &DeviceInfo, bus_id: u8, device_id: u8) -> bool {
    if bus_id == 0xff {
        // no filter
        return true;
    }
    devicepath.busnum() == bus_id && devicepath.device_address() != device_id
}
