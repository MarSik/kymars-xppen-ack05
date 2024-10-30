use enumset::{EnumSet, EnumSetType};
use hidapi::{self, BusType, HidApi, HidDevice, HidResult};

use crate::kbd_events::HasState;
use crate::layout::types::KeyCoords;

const PID: u16 = 0x0202;
const VID: u16 = 0x28bd;

// XP-Pen ACK05
pub struct XpPenAck05 {
    device: HidDevice,
}

#[derive(EnumSetType, Debug, Hash)]
pub enum XpPenButtons {
    XpB01,
    XpB02,
    XpB03,
    XpB04,
    XpB05,
    XpB06,
    XpB07,
    XpB08,
    XpB09,
    XpB10,
    XpRoCW,
    XpRoCCW,
}

impl Into<KeyCoords> for XpPenButtons {
    fn into(self) -> KeyCoords {
        return KeyCoords(0, 0, self as u8);
    }
}

impl HasState for XpPenButtons {
    // Rotary encoder has no state, all the other buttons can be up or down
    // Stateless buttons emit a pressed event every time they appear in the pressed report
    fn has_state(self) -> bool {
        return !(self == XpPenButtons::XpRoCCW || self == XpPenButtons::XpRoCW);
    }
}

fn open_keyboard(api: &HidApi) -> Option<HidDevice> {
    for device in api.device_list() {
        if device.vendor_id() == VID
            && device.product_id() == PID
            && device.usage_page() == 0xff0a
            && device.usage() == 0x1
        {
            println!(
                "SELECTING {:?} {:?} {:?} {:?} interface: {} usage: {:04x} ({:04x})",
                device.path(),
                device.manufacturer_string(),
                device.product_string(),
                device.serial_number(),
                device.interface_number(),
                device.usage(),
                device.usage_page()
            );
            if let HidResult::Ok(hid) = device.open_device(api) {
                return Some(hid);
            }
        }
    }

    println!("No device found.");
    None
}

#[derive(Debug, Clone, Copy)]
pub enum XpPenResult {
    Timeout,
    TryAgain,
    Keys(EnumSet<XpPenButtons>),
}

impl XpPenAck05 {
    pub fn new() -> Self {
        let api = hidapi::HidApi::new().unwrap();

        // Print out information about all connected devices
        for device in api.device_list() {
            println!(
                "0x{:04x}:0x{:04x} 0x{:04x}:0x{:04x} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}",
                device.vendor_id(),
                device.product_id(),
                device.usage(),
                device.usage_page(),
                device,
                device.manufacturer_string(),
                device.product_string(),
                device.serial_number(),
                device.interface_number(),
                device.bus_type(),
                device.release_number(),
                device.path()
            );
        }

        // Connect to device using its VID and PID
        let device = open_keyboard(&api).unwrap();
        println!("Device: {:?}", device);

        // Initialize XP-Pen ACK05
        // This was sniffed from the USB communication between the official application
        // and the device. It switches the protocol to represent each key with one bit
        // instead of sending HID scan codes.
        let bus = device
            .get_device_info()
            .map_or(BusType::Usb, |info| info.bus_type());
        if let BusType::Usb = bus {
            println!("Configuring USB HID key bit mode.");
            let buf = [0x02, 0xb0, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
            let res = device.write(&buf).unwrap();
            println!("Wrote: {:?} byte(s)", res);
        } else if let BusType::Bluetooth = bus {
            println!("Configuring Bluetooth HID key bit mode.");
            panic!("Bluetooth connection is currently not supported!.");
            //let buf = [0x02, 0xb0, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
            //let res = device.write(&buf).unwrap();
            //println!("Wrote: {:?} byte(s)", res);
        }

        Self { device }
    }

    pub fn set_blocking(&self) {
        let _ = self.device.set_blocking_mode(true);
    }

    pub fn read(&self, block: bool) -> XpPenResult {
        let mut buf = [0u8; 32];

        let timeout = if block { -1 } else { 25 };

        let res = self.device.read_timeout(&mut buf[..], timeout).unwrap();
        //println!("Read: {:?}", &buf[..res]);
        if res == 0 {
            return XpPenResult::Timeout;
        }

        if buf[1] != 240 {
            return XpPenResult::TryAgain;
        }

        let mut state = EnumSet::empty();

        if buf[2] & 0x01 > 0 {
            state |= XpPenButtons::XpB01;
        }
        if buf[2] & 0x02 > 0 {
            state |= XpPenButtons::XpB02;
        }
        if buf[2] & 0x04 > 0 {
            state |= XpPenButtons::XpB03;
        }
        if buf[2] & 0x08 > 0 {
            state |= XpPenButtons::XpB04;
        }
        if buf[2] & 0x10 > 0 {
            state |= XpPenButtons::XpB05;
        }
        if buf[2] & 0x20 > 0 {
            state |= XpPenButtons::XpB06;
        }
        if buf[2] & 0x40 > 0 {
            state |= XpPenButtons::XpB07;
        }
        if buf[2] & 0x80 > 0 {
            state |= XpPenButtons::XpB08;
        }
        if buf[3] & 0x01 > 0 {
            state |= XpPenButtons::XpB09;
        }
        if buf[3] & 0x02 > 0 {
            state |= XpPenButtons::XpB10;
        }
        if buf[7] & 0x01 > 0 {
            state |= XpPenButtons::XpRoCW;
        }
        if buf[7] & 0x02 > 0 {
            state |= XpPenButtons::XpRoCCW;
        }

        return XpPenResult::Keys(state);
    }
}
