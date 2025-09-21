use futures_lite::future::block_on;
use log::{info, trace};
use nusb::transfer::{ControlOut, ControlType, Queue, Recipient, RequestBuffer};
use nusb::{Device, DeviceInfo, Interface};
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
        let mut queue = self.interface.bulk_in_queue(BULK_EP_IN);

        block_on(self.interface.bulk_out(BULK_EP_OUT, to_device))
            .into_result()
            .unwrap();

        trace!("  awaiting answer");
        let mut answer = self.read_answer(&mut queue);

        let payloadsize: u16 = u16::from_be_bytes(answer[1..3].try_into().unwrap());
        while answer.len() < payloadsize as usize + 4 {
            trace!("  waiting for more data");
            answer.append(&mut self.read_answer(&mut queue));
        }

        return answer;
    }

    fn cmd_oneway_devicereset(&mut self, to_device: Vec<u8>) {
        block_on(self.interface.bulk_out(BULK_EP_OUT, to_device))
            .into_result()
            .unwrap();

        info!("  TODO: wait for device reset");
        // TODO, maybe with nusb 0.2: try to make sure the new device is on the same path
        let (device, device_info, interface) =
            Self::setup_device_and_interface(true, self.bus_id, self.device_id);
        self.device = device;
        self.interface = interface;
        self.bus_id = device_info.bus_number();
        self.device_id = device_info.device_address();
    }

    fn get_time_micros(&self) -> u64 {
        let duration_since_epoch = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let timestamp_micros = duration_since_epoch.as_micros();
        return timestamp_micros as u64;
    }
}

impl IntfBulk {
    pub fn new() -> Self {
        let (device, device_info, interface) = Self::setup_device_and_interface(false, 0xff, 0xff);
        Self {
            device: device,
            interface: interface,
            bus_id: device_info.bus_number(),
            device_id: device_info.device_address(),
        }
    }

    fn wait_for_deviceinfo(wait: bool, bus_id: u8, device_id: u8) -> DeviceInfo {
        let mut sleep_time = 1000;
        loop {
            let di_opt = nusb::list_devices().unwrap().find(|d| {
                d.vendor_id() == DEVID_VENDOR
                    && d.product_id() == DEVID_PRODUCT
                    && match_partially_last_device(&d, bus_id, device_id)
            });

            if di_opt.is_some() {
                return di_opt.unwrap();
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

        let mut device = di.open().unwrap();
        let interface = device.detach_and_claim_interface(DEVICE_INTERFACE).unwrap();

        // set control line state request - needed for the device to reply in BULK mode
        //device.control_out_blocking(handle, 0x21, 0x22 /* set line state*/, 3, 0, NULL, 0, 2000);

        Self::ctrl_set_line_state(&mut device);
        return (device, di, interface);
    }

    fn read_answer(&mut self, in_queue: &mut Queue<RequestBuffer>) -> Vec<u8> {
        loop {
            while in_queue.pending() < 8 {
                in_queue.submit(RequestBuffer::new(256));
            }
            let result = block_on(in_queue.next_complete());

            result.status.expect("Error while reading from USB");

            return result.data;
        }
    }

    fn ctrl_set_line_state(device: &mut Device) {
        println!("Send ctrl_set_line_state");
        block_on(device.control_out(ControlOut {
            control_type: ControlType::Class,
            recipient: Recipient::Device,
            request: 0x22, /* set line state*/
            value: 0x03,
            index: 0x00,
            data: &[],
        }))
        .into_result()
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
    devicepath.bus_number() == bus_id && devicepath.device_address() != device_id
}
