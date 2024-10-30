use std::thread::sleep;
use std::time::{self, Duration};

use xppen_ack05::layout::switcher::LayerSwitcher;
use xppen_ack05::xppen_hid::{XpPenAck05, XpPenResult};
use xppen_ack05::virtual_keyboard::VirtualKeyboard;
use xppen_ack05::kbd_events::ChangeDetector;
use xppen_ack05::layout::serialization::load_layout;


fn main() {
    // Open XPPen ACK05
    let xppen = XpPenAck05::new();

    // XPPen State machine
    let mut xppen_events = ChangeDetector::new();

    let layout = load_layout("test");
    let mut layout_runtime = LayerSwitcher::new(&layout);
    layout_runtime.start();

    // Create a virtual keyboard
    let mut kbd = VirtualKeyboard::new(layout_runtime.get_used_keys());

    // Wait for a HID event when reading from XP Pen (= block)
    xppen.set_blocking();

    loop {
        // Read state data from device
        // When any button is pressed use read timeout so the long press can be
        // analyzed in between messages.
        let result = xppen.read(!xppen_events.has_short_pressed());
        //println!("{:?}", result);

        if let XpPenResult::Keys(buttons) = result {
            // Compute state changes
            xppen_events.analyze(buttons, time::Instant::now());
        } else {
            xppen_events.tick(time::Instant::now());
        }

        // Emit virtual keys
        while let Some(ev) = xppen_events.next() {
            println!("Input: {:?}", ev);
            layout_runtime.process_keyevent(ev, time::Instant::now());
            layout_runtime.render(|k, s| {
                println!("Output > {:?} pressed {}", k, s);
                kbd.emit_key(k, s);
                sleep(Duration::from_millis(2));
            });
        }
    }
}
