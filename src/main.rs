use std::thread::sleep;
use std::time::{self, Duration};

use xppen_act05::xppen_hid::XpPenAct05;
use xppen_act05::virtual_keyboard::VirtualKeyboard;
use xppen_act05::kbd_events::ChangeDetector;
use xppen_act05::layout::serialization::load_layout;


fn main() {
    // Open XPPen ACT05
    let xppen = XpPenAct05::new();

    // XPPen State machine
    let mut xppen_events = ChangeDetector::new();

    let mut layout = load_layout("test");
    layout.start();

    // Create a virtual keyboard
    let mut kbd = VirtualKeyboard::new(layout.get_used_keys());

    // Wait for a HID event when reading from XP Pen (= block)
    xppen.set_blocking();

    loop {
        // Read state data from device
        let buttons = xppen.read();
        //println!("{:?}", buttons);

        // Compute state changes
        xppen_events.analyze(buttons);

        // Emit virtual keys
        while let Some(ev) = xppen_events.next() {
            println!("Input: {:?}", ev);
            layout.process_keyevent(ev, time::Instant::now());
            layout.render(|k, s| {
                println!("Output > {:?} pressed {}", k, s);
                kbd.emit_key(k, s);
                sleep(Duration::from_millis(2));
            });
        }
    }
}
