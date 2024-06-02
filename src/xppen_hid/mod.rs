use enumset::{EnumSet, EnumSetType};
use hidapi::{self, DeviceInfo, HidApi, HidDevice};

use crate::kbd_events::HasState;
use crate::layout::types::KeyCoords;

const PID: u16 = 0x0202;
const VID: u16 = 0x28bd;

// XP-Pen ACT05
pub struct XpPenAct05 {
    device: HidDevice
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
    XpRoCCW
}

impl Into<KeyCoords> for XpPenButtons {
    fn into(self) -> KeyCoords {
        return KeyCoords(0, 0, self as u8)
    }
}

impl HasState for XpPenButtons {
    // Rotary encoder has no state, all the other buttons can be up or down
    // Stateless buttons emit a pressed event every time they appear in the pressed report
    fn has_state(self) -> bool {
        return !(self == XpPenButtons::XpRoCCW || self == XpPenButtons::XpRoCW);
    }
}


fn open_keyboard(api: &HidApi) -> Option<&DeviceInfo> {
    for device in api.device_list() {
        if device.vendor_id() == VID && device.product_id() == PID && device.interface_number() == 2 {
            println!("SELECTING {:?} {:?} {:?} {:?}", device, device.manufacturer_string(), device.product_string(), device.serial_number());
            return Some(device)
        }
    }

    None
}


impl XpPenAct05 {
    pub fn new() -> Self {
        let api = hidapi::HidApi::new().unwrap();

        // Print out information about all connected devices
        //for device in api.device_list() {
        //    println!("{:?} {:?} {:?} {:?} {:?}", device, device.manufacturer_string(), device.product_string(), device.serial_number(), device.interface_number());
        //}

        // Connect to device using its VID and PID
        let device = open_keyboard(&api).unwrap().open_device(&api).unwrap();
        println!("Device: {:?}", device);

        // Initialize XP-Pen ACT05
        // This was sniffed from the USB communication between the official application
        // and the device. It switches the protocol to represent each key with one bit
        // instead of sending HID scan codes.
        let buf = [0x02, 0xb0, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let res = device.write(&buf).unwrap();
        println!("Wrote: {:?} byte(s)", res);

        Self {
            device
        }
    }

    pub fn set_blocking(&self) {
        let _= self.device.set_blocking_mode(true);
    }

    pub fn read(&self) -> EnumSet<XpPenButtons> {
        let mut buf = [0u8; 32];
        let res = self.device.read(&mut buf[..]).unwrap();
        //println!("Read: {:?}", &buf[..res]);

        if buf[1] != 240 {
            return EnumSet::empty();
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

        return state;
    }
}
