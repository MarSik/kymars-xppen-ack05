use evdev::{AttributeSet, EventType, InputEvent, Key};
use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};

pub struct VirtualKeyboard {
    kbd: VirtualDevice
}

impl VirtualKeyboard {
    pub fn new<I>(keyset: I) -> Self
    where
        I: IntoIterator<Item=Key>
    {
        let mut keys = AttributeSet::<Key>::new();
        for k in keyset {
            keys.insert(k);
        }

        let mut kbd = VirtualDeviceBuilder::new().unwrap()
            .name("XP-Pen ACK05 driver")
            .with_keys(&keys).unwrap()
            .build()
            .unwrap();

        for path in kbd.enumerate_dev_nodes_blocking().unwrap() {
            let path = path.unwrap();
            println!("Available as {}", path.display());
        }

        Self {
            kbd
        }
    }

    pub fn emit_key(&mut self, key: Key, down: bool) {
        let code = key.code();
        let type_ = EventType::KEY;

        if down {
            let down_event = InputEvent::new(type_, code, 1);
            self.kbd.emit(&[down_event]).unwrap();
        } else {
            let down_event = InputEvent::new(type_, code, 0);
            self.kbd.emit(&[down_event]).unwrap();
        }
    }
}
