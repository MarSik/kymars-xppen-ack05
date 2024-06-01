use evdev::Key;

use crate::kbd_events::KeyStateChange;
use crate::layout::layer::Layer;
use crate::layout::types::KeyCoords;
use crate::layout::switcher::LayerSwitcher;
use crate::layout::types::KeymapEvent::{K, Kms, No, Lhold, Inh, Ltap, Lactivate, Pass, LhtK, LhtKg, LhtL};

use self::testtime::TestTime;

#[non_exhaustive]
struct TestDevice;

impl TestDevice {
    pub(crate) const B01: KeyCoords = KeyCoords(0, 0, 0);
    pub(crate) const B02: KeyCoords = KeyCoords(0, 0, 1);
    pub(crate) const B03: KeyCoords = KeyCoords(0, 1, 0);
    pub(crate) const B04: KeyCoords = KeyCoords(0, 1, 1);
}

const DEFAULT_LAYER_CONFIG: Layer = Layer{
    status_on_reset: crate::layout::types::LayerStatus::LayerActive,
    inherit: None,
    on_active_keys: vec![],
    disable_active_on_press: false,
    on_timeout_layer: None,
    timeout: None,
    keymap: vec![],
    default_action: crate::layout::types::KeymapEvent::Pass,
};

#[track_caller]
fn assert_emitted_keys(layout: &mut LayerSwitcher, keys: Vec<(Key, bool)>) {
    let mut received = Vec::new();

    // The test could be done directly in the closure, but the asserts then
    // report a wrong caller line, because track_caller is still unstable
    // for closures.
    layout.render(|k, v| {
        received.push((k, v));
    });

    let mut idx = 0;
    for (k, v) in received {
        assert!(idx < keys.len(), "Unexpected key {:?}/{}", k, v);
        assert_eq!(keys[idx].0, k, "Expected key {:?}/{} got {:?}/{}", keys[idx].0, keys[idx].1, k, v);
        assert_eq!(keys[idx].1, v, "Expected key {:?} state to be {} got {}", k, keys[idx].1, v);
        idx += 1;
    }

    assert_eq!(idx, keys.len(), "Expected {} key presses. Got only {}.", keys.len(), idx);
}

// Single layer, basic key press and release test
fn basic_layout() -> LayerSwitcher {
    let keymap_default = vec![ // blocks
        vec![ // rows
            vec![ K(Key::KEY_LEFTALT),   K(Key::KEY_B) ],
            vec![ K(Key::KEY_LEFTSHIFT), No,           ],
        ],
    ];

    let default_layer = Layer{
        keymap: keymap_default,
        ..DEFAULT_LAYER_CONFIG
    };

    let layers = vec![default_layer];

    LayerSwitcher::new(layers)
}

mod testtime;

#[test]
fn test_basic_layout() {
    let mut layout = basic_layout();
    layout.start();

    let mut t = TestTime::start();

    assert_emitted_keys(&mut layout, vec![]);

    layout.process_keyevent(KeyStateChange::Pressed(TestDevice::B01), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_LEFTALT, true)]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B02), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_B, true), (Key::KEY_B, false)]);

    layout.process_keyevent(KeyStateChange::Released(TestDevice::B01), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_LEFTALT, false)]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B04), t.advance_ms(10));
    assert_emitted_keys(&mut layout, vec![]);
}

// Dual layout, basic test simulating Shift behavior (hold to stay in the second layer)
// It also tests pass-through to lower layer and inheritance from inactive layer
fn basic_layered_layout() -> LayerSwitcher {
    let keymap_default = vec![ // blocks
        vec![ // rows
            vec![ Lhold(1),              K(Key::KEY_B) ],
            vec![ K(Key::KEY_LEFTSHIFT), No,           ],
        ],
    ];

    let keymap_shift = vec![ // blocks
        vec![ // rows
            vec![ K(Key::KEY_0), Pass,          ],
            vec![ Inh          , K(Key::KEY_E), ],
        ],
    ];

    let keymap_inh = vec![ // blocks
        vec![ // rows
            vec![ K(Key::KEY_1), K(Key::KEY_9), ],
            vec![ K(Key::KEY_2), K(Key::KEY_3), ],
        ],
    ];

    let default_layer = Layer{
        keymap: keymap_default,
        ..DEFAULT_LAYER_CONFIG
    };

    let shift_layer = Layer{
        status_on_reset: crate::layout::types::LayerStatus::LayerPassthrough,
        inherit: Some(2),
        on_active_keys: vec![Key::KEY_LEFTSHIFT],
        keymap: keymap_shift,
        ..DEFAULT_LAYER_CONFIG
    };

    let inh_layer = Layer{
        status_on_reset: crate::layout::types::LayerStatus::LayerDisabled,
        keymap: keymap_inh,
        ..DEFAULT_LAYER_CONFIG
    };

    let layers = vec![default_layer, shift_layer, inh_layer];

    LayerSwitcher::new(layers)
}

#[test]
fn test_basic_layered_layout() {
    let mut layout = basic_layered_layout();
    layout.start();

    let mut t = TestTime::start();

    assert_emitted_keys(&mut layout, vec![]);

    layout.process_keyevent(KeyStateChange::Pressed(TestDevice::B01), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_LEFTSHIFT, true)]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B02), t.advance_ms(1));
    assert_emitted_keys(&mut layout, vec![(Key::KEY_B, true), (Key::KEY_B, false)]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B04), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_E, true), (Key::KEY_E, false)]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B03), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_2, true), (Key::KEY_2, false)]);

    layout.process_keyevent(KeyStateChange::Released(TestDevice::B01), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_LEFTSHIFT, false)]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B04), t);
    assert_emitted_keys(&mut layout, vec![]);
}

#[test]
fn test_basic_layered_layout_cross_release() {
    let mut layout = basic_layered_layout();
    layout.start();
    let mut t = TestTime::start();

    assert_emitted_keys(&mut layout, vec![]);

    layout.process_keyevent(KeyStateChange::Pressed(TestDevice::B01), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_LEFTSHIFT, true)]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B02), t.advance_ms(1));
    assert_emitted_keys(&mut layout, vec![(Key::KEY_B, true), (Key::KEY_B, false)]);

    layout.process_keyevent(KeyStateChange::Pressed(TestDevice::B04), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_E, true),]);

    layout.process_keyevent(KeyStateChange::Released(TestDevice::B01), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_LEFTSHIFT, false)]);

    layout.process_keyevent(KeyStateChange::Released(TestDevice::B04), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_E, false)]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B04), t);
    assert_emitted_keys(&mut layout, vec![]);
}

// Dual layout, basic test simulating dead-key (sticky) behavior (stay in the second layer until next key is pressed)
fn tap_layered_layout() -> LayerSwitcher {
    let keymap_default = vec![ // blocks
        vec![ // rows
            vec![ Ltap(1),               K(Key::KEY_B) ],
            vec![ K(Key::KEY_LEFTSHIFT), No,           ],
        ],
    ];

    let keymap_shift = vec![ // blocks
        vec![ // rows
            vec![ No,                    Inh,           ],
            vec![ K(Key::KEY_LEFTSHIFT), K(Key::KEY_E), ],
        ],
    ];

    let default_layer = Layer{
        keymap: keymap_default,
        ..DEFAULT_LAYER_CONFIG
    };

    let shift_layer = Layer{
        status_on_reset: crate::layout::types::LayerStatus::LayerPassthrough,
        on_active_keys: vec![Key::KEY_LEFTSHIFT],
        keymap: keymap_shift,
        ..DEFAULT_LAYER_CONFIG
    };

    let layers = vec![default_layer, shift_layer];

    LayerSwitcher::new(layers)
}

#[test]
fn test_tap_layered_layout() {
    let mut layout = tap_layered_layout();
    layout.start();
    let mut t = TestTime::start();

    assert_emitted_keys(&mut layout, vec![]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B01), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_LEFTSHIFT, true)]);

    assert_eq!(layout.get_active_layers(), vec![0, 1]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B02), t.advance_ms(1));
    assert_emitted_keys(&mut layout, vec![(Key::KEY_B, true), (Key::KEY_LEFTSHIFT, false), (Key::KEY_B, false)]);

    assert_eq!(layout.get_active_layers(), vec![0]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B04), t);
    assert_emitted_keys(&mut layout, vec![]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B04), t);
    assert_emitted_keys(&mut layout, vec![]);
}

#[test]
fn test_tap_layered_hold() {
    let mut layout = tap_layered_layout();
    layout.start();
    let mut t = TestTime::start();

    assert_emitted_keys(&mut layout, vec![]);

    layout.process_keyevent(KeyStateChange::Pressed(TestDevice::B01), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_LEFTSHIFT, true)]);

    assert_eq!(layout.get_active_layers(), vec![0, 1]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B02), t.advance_ms(1));
    assert_emitted_keys(&mut layout, vec![(Key::KEY_B, true), (Key::KEY_B, false) ]);

    assert_eq!(layout.get_active_layers(), vec![0, 1]);

    layout.process_keyevent(KeyStateChange::Released(TestDevice::B01), t);
    assert_emitted_keys(&mut layout, vec![]);

    assert_eq!(layout.get_active_layers(), vec![0, 1]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B04), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_E, true), (Key::KEY_LEFTSHIFT, false), (Key::KEY_E, false)]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B04), t);
    assert_emitted_keys(&mut layout, vec![]);
}

#[test]
fn test_tap_layered_hold_crossed() {
    let mut layout = tap_layered_layout();
    layout.start();
    let mut t = TestTime::start();

    assert_emitted_keys(&mut layout, vec![]);

    layout.process_keyevent(KeyStateChange::Pressed(TestDevice::B01), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_LEFTSHIFT, true)]);

    assert_eq!(layout.get_active_layers(), vec![0, 1]);

    layout.process_keyevent(KeyStateChange::Pressed(TestDevice::B02), t.advance_ms(1));
    assert_emitted_keys(&mut layout, vec![(Key::KEY_B, true) ]);

    assert_eq!(layout.get_active_layers(), vec![0, 1]);

    layout.process_keyevent(KeyStateChange::Released(TestDevice::B01), t);
    assert_emitted_keys(&mut layout, vec![]);

    assert_eq!(layout.get_active_layers(), vec![0, 1]);

    layout.process_keyevent(KeyStateChange::Released(TestDevice::B02), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_B, false) ]);

    assert_eq!(layout.get_active_layers(), vec![0, 1]);

    layout.process_keyevent(KeyStateChange::Pressed(TestDevice::B04), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_E, true), (Key::KEY_LEFTSHIFT, false)]);

    assert_eq!(layout.get_active_layers(), vec![0]);

    layout.process_keyevent(KeyStateChange::Released(TestDevice::B04), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_E, false)]);
}

#[test]
fn test_tap_layered_hold_dual_crossed() {
    let mut layout = tap_layered_layout();
    layout.start();
    let mut t = TestTime::start();

    assert_emitted_keys(&mut layout, vec![]);

    layout.process_keyevent(KeyStateChange::Pressed(TestDevice::B01), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_LEFTSHIFT, true)]);

    assert_eq!(layout.get_active_layers(), vec![0, 1]);

    layout.process_keyevent(KeyStateChange::Pressed(TestDevice::B02), t.advance_ms(1));
    assert_emitted_keys(&mut layout, vec![(Key::KEY_B, true) ]);

    assert_eq!(layout.get_active_layers(), vec![0, 1]);

    layout.process_keyevent(KeyStateChange::Released(TestDevice::B01), t);
    assert_emitted_keys(&mut layout, vec![]);

    assert_eq!(layout.get_active_layers(), vec![0, 1]);

    layout.process_keyevent(KeyStateChange::Pressed(TestDevice::B04), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_E, true), (Key::KEY_LEFTSHIFT, false)]);

    assert_eq!(layout.get_active_layers(), vec![0]);

    layout.process_keyevent(KeyStateChange::Released(TestDevice::B02), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_B, false) ]);

    assert_eq!(layout.get_active_layers(), vec![0]);

    layout.process_keyevent(KeyStateChange::Released(TestDevice::B04), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_E, false)]);
}

#[test]
fn test_tap_layered_hold_dual_crossed_lifo() {
    let mut layout = tap_layered_layout();
    layout.start();
    let mut t = TestTime::start();

    assert_emitted_keys(&mut layout, vec![]);

    layout.process_keyevent(KeyStateChange::Pressed(TestDevice::B01), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_LEFTSHIFT, true)]);

    assert_eq!(layout.get_active_layers(), vec![0, 1]);

    layout.process_keyevent(KeyStateChange::Pressed(TestDevice::B02), t.advance_ms(1));
    assert_emitted_keys(&mut layout, vec![(Key::KEY_B, true) ]);

    assert_eq!(layout.get_active_layers(), vec![0, 1]);

    layout.process_keyevent(KeyStateChange::Released(TestDevice::B01), t);
    assert_emitted_keys(&mut layout, vec![]);

    assert_eq!(layout.get_active_layers(), vec![0, 1]);

    layout.process_keyevent(KeyStateChange::Pressed(TestDevice::B04), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_E, true), (Key::KEY_LEFTSHIFT, false)]);

    assert_eq!(layout.get_active_layers(), vec![0]);

    layout.process_keyevent(KeyStateChange::Released(TestDevice::B04), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_E, false)]);

    assert_eq!(layout.get_active_layers(), vec![0]);

    layout.process_keyevent(KeyStateChange::Released(TestDevice::B02), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_B, false) ]);
}

// Dual layout, basic test simulating Shift behavior (hold to stay in the second layer),
// but with a key in second layer disabling shift temporarily
fn layered_layout_with_masked_key() -> LayerSwitcher {
    let keymap_default = vec![ // blocks
        vec![ // rows
            vec![ Lhold(1),              K(Key::KEY_B) ],
            vec![ K(Key::KEY_LEFTSHIFT), No,           ],
        ],
    ];

    let keymap_shift = vec![ // blocks
        vec![ // rows
            vec![ K(Key::KEY_0),         Inh,           ],
            vec![ K(Key::KEY_LEFTSHIFT), Kms(vec![Key::KEY_LEFTSHIFT], vec![Key::KEY_E]), ],
        ],
    ];

    let default_layer = Layer{
        keymap: keymap_default,
        ..DEFAULT_LAYER_CONFIG
    };

    let shift_layer = Layer{
        status_on_reset: crate::layout::types::LayerStatus::LayerPassthrough,
        on_active_keys: vec![Key::KEY_LEFTSHIFT],
        keymap: keymap_shift,
        ..DEFAULT_LAYER_CONFIG
    };

    let layers = vec![default_layer, shift_layer];

    LayerSwitcher::new(layers)
}

#[test]
fn test_layered_layout_w_masked_key() {
    let mut layout = layered_layout_with_masked_key();
    layout.start();
    let mut t = TestTime::start();

    assert_emitted_keys(&mut layout, vec![]);

    layout.process_keyevent(KeyStateChange::Pressed(TestDevice::B01), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_LEFTSHIFT, true)]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B02), t.advance_ms(1));
    assert_emitted_keys(&mut layout, vec![(Key::KEY_B, true), (Key::KEY_B, false)]);

    // This temporarily masks the Shift key
    layout.process_keyevent(KeyStateChange::Click(TestDevice::B04), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_LEFTSHIFT, false), (Key::KEY_E, true), (Key::KEY_E, false), (Key::KEY_LEFTSHIFT, true)]);

    layout.process_keyevent(KeyStateChange::Released(TestDevice::B01), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_LEFTSHIFT, false)]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B04), t);
    assert_emitted_keys(&mut layout, vec![]);
}


// Dual layout, basic test simulating Shift behavior (hold to stay in the second layer),
// but with the second layer disabling active keys on press
fn layered_layout_with_mask() -> LayerSwitcher {
    let keymap_default = vec![ // blocks
        vec![ // rows
            vec![ Lhold(1),              K(Key::KEY_B) ],
            vec![ K(Key::KEY_LEFTSHIFT), No,           ],
        ],
    ];

    let keymap_shift = vec![ // blocks
        vec![ // rows
            vec![ K(Key::KEY_0),         Inh,           ],
            vec![ K(Key::KEY_LEFTSHIFT), K(Key::KEY_E), ],
        ],
    ];

    let default_layer = Layer{
        keymap: keymap_default,
        ..DEFAULT_LAYER_CONFIG
    };

    let shift_layer = Layer{
        status_on_reset: crate::layout::types::LayerStatus::LayerPassthrough,
        on_active_keys: vec![Key::KEY_LEFTSHIFT],
        disable_active_on_press: true,
        keymap: keymap_shift,
        ..DEFAULT_LAYER_CONFIG
    };

    let layers = vec![default_layer, shift_layer];

    LayerSwitcher::new(layers)
}


#[test]
fn test_layered_layout_w_mask() {
    let mut layout = layered_layout_with_mask();
    layout.start();
    let mut t = TestTime::start();

    assert_emitted_keys(&mut layout, vec![]);

    layout.process_keyevent(KeyStateChange::Pressed(TestDevice::B01), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_LEFTSHIFT, true)]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B02), t.advance_ms(1));
    assert_emitted_keys(&mut layout, vec![(Key::KEY_B, true), (Key::KEY_B, false)]);

    // This temporarily masks the Shift key
    layout.process_keyevent(KeyStateChange::Pressed(TestDevice::B04), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_LEFTSHIFT, false), (Key::KEY_E, true)]);

    layout.process_keyevent(KeyStateChange::Released(TestDevice::B04), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_E, false), (Key::KEY_LEFTSHIFT, true)]);

    layout.process_keyevent(KeyStateChange::Released(TestDevice::B01), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_LEFTSHIFT, false)]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B04), t);
    assert_emitted_keys(&mut layout, vec![]);
}

#[test]
fn test_layered_layout_w_mask_crossed() {
    let mut layout = layered_layout_with_mask();
    layout.start();
    let mut t = TestTime::start();

    assert_emitted_keys(&mut layout, vec![]);

    layout.process_keyevent(KeyStateChange::Pressed(TestDevice::B01), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_LEFTSHIFT, true)]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B02), t.advance_ms(1));
    assert_emitted_keys(&mut layout, vec![(Key::KEY_B, true), (Key::KEY_B, false)]);

    // This temporarily masks the Shift key
    layout.process_keyevent(KeyStateChange::Pressed(TestDevice::B04), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_LEFTSHIFT, false), (Key::KEY_E, true)]);

    layout.process_keyevent(KeyStateChange::Released(TestDevice::B01), t);
    assert_emitted_keys(&mut layout, vec![]);

    layout.process_keyevent(KeyStateChange::Released(TestDevice::B04), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_E, false)]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B04), t);
    assert_emitted_keys(&mut layout, vec![]);
}

// Dual layout, basic test simulating hold layer with timeout behavior
fn hold_and_tap_layered_layout() -> LayerSwitcher {
    let keymap_default = vec![ // blocks
        vec![ // rows
            vec![ LhtL(1, 2),            K(Key::KEY_B) ],
            vec![ K(Key::KEY_LEFTSHIFT), No,           ],
        ],
    ];

    let keymap_shift = vec![ // blocks
        vec![ // rows
            vec![ No,                    K(Key::KEY_T),           ],
            vec![ K(Key::KEY_LEFTSHIFT), K(Key::KEY_E), ],
        ],
    ];

    let keymap_tap = vec![ // blocks
        vec![ // rows
            vec![ No,            K(Key::KEY_3), ],
            vec![ K(Key::KEY_1), K(Key::KEY_2), ],
        ],
    ];

    let default_layer = Layer{
        keymap: keymap_default,
        ..DEFAULT_LAYER_CONFIG
    };

    let shift_layer = Layer{
        status_on_reset: crate::layout::types::LayerStatus::LayerPassthrough,
        keymap: keymap_shift,
        ..DEFAULT_LAYER_CONFIG
    };

    let tap_layer = Layer{
        status_on_reset: crate::layout::types::LayerStatus::LayerPassthrough,
        keymap: keymap_tap,
        ..DEFAULT_LAYER_CONFIG
    };

    let layers = vec![default_layer, shift_layer, tap_layer];

    LayerSwitcher::new(layers)
}

#[test]
fn test_hold_and_tap_layered_layout() {
    let mut layout = hold_and_tap_layered_layout();
    layout.start();
    let mut t = TestTime::start();

    assert_emitted_keys(&mut layout, vec![]);

    layout.process_keyevent(KeyStateChange::Pressed(TestDevice::B01), t);
    assert_emitted_keys(&mut layout, vec![]);

    assert_eq!(layout.get_active_layers(), vec![0, 1]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B02), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_T, true), (Key::KEY_T, false)]);

    assert_eq!(layout.get_active_layers(), vec![0, 1]);

    layout.process_keyevent(KeyStateChange::Released(TestDevice::B01), t.advance_ms(190));
    assert_emitted_keys(&mut layout, vec![]);

    // Time was short enough for tap switch
    assert_eq!(layout.get_active_layers(), vec![0, 2]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B04), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_2, true), (Key::KEY_2, false)]);

    assert_eq!(layout.get_active_layers(), vec![0]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B04), t);
    assert_emitted_keys(&mut layout, vec![]);
}

#[test]
fn test_hold_and_tap_layered_layout_long_press() {
    let mut layout = hold_and_tap_layered_layout();
    layout.start();
    let mut t = TestTime::start();

    assert_emitted_keys(&mut layout, vec![]);

    layout.process_keyevent(KeyStateChange::Pressed(TestDevice::B01), t);
    assert_emitted_keys(&mut layout, vec![]);

    assert_eq!(layout.get_active_layers(), vec![0, 1]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B02), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_T, true), (Key::KEY_T, false)]);

    assert_eq!(layout.get_active_layers(), vec![0, 1]);

    layout.process_keyevent(KeyStateChange::Released(TestDevice::B01), t.advance_ms(220));
    assert_emitted_keys(&mut layout, vec![]);

    // Time was too long for a tap switch
    assert_eq!(layout.get_active_layers(), vec![0]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B04), t);
    assert_emitted_keys(&mut layout, vec![]);
}

// Dual layout, basic test simulating hold layer with key timeout behavior
fn hold_and_tap_key_layered_layout() -> LayerSwitcher {
    let keymap_default = vec![ // blocks
        vec![ // rows
            vec![ LhtK(1, Key::KEY_0),   K(Key::KEY_B) ],
            vec![ K(Key::KEY_LEFTSHIFT), No,           ],
        ],
    ];

    let keymap_shift = vec![ // blocks
        vec![ // rows
            vec![ No,                    K(Key::KEY_T), ],
            vec![ K(Key::KEY_LEFTSHIFT), K(Key::KEY_E), ],
        ],
    ];

    let default_layer = Layer{
        keymap: keymap_default,
        ..DEFAULT_LAYER_CONFIG
    };

    let shift_layer = Layer{
        status_on_reset: crate::layout::types::LayerStatus::LayerPassthrough,
        keymap: keymap_shift,
        ..DEFAULT_LAYER_CONFIG
    };

    let layers = vec![default_layer, shift_layer];

    LayerSwitcher::new(layers)
}

#[test]
fn test_hold_and_tap_key_layered_layout() {
    let mut layout = hold_and_tap_key_layered_layout();
    layout.start();
    let mut t = TestTime::start();

    assert_emitted_keys(&mut layout, vec![]);

    layout.process_keyevent(KeyStateChange::Pressed(TestDevice::B01), t);
    assert_emitted_keys(&mut layout, vec![]);

    assert_eq!(layout.get_active_layers(), vec![0, 1]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B02), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_T, true), (Key::KEY_T, false)]);

    assert_eq!(layout.get_active_layers(), vec![0, 1]);

    // Time was short enough for tap key
    layout.process_keyevent(KeyStateChange::Released(TestDevice::B01), t.advance_ms(190));
    assert_emitted_keys(&mut layout, vec![(Key::KEY_0, true), (Key::KEY_0, false)]);

    assert_eq!(layout.get_active_layers(), vec![0]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B04), t);
    assert_emitted_keys(&mut layout, vec![]);
}

#[test]
fn test_hold_and_tap_key_layered_layout_long_press() {
    let mut layout = hold_and_tap_key_layered_layout();
    layout.start();
    let mut t = TestTime::start();

    assert_emitted_keys(&mut layout, vec![]);

    layout.process_keyevent(KeyStateChange::Pressed(TestDevice::B01), t);
    assert_emitted_keys(&mut layout, vec![]);

    assert_eq!(layout.get_active_layers(), vec![0, 1]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B02), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_T, true), (Key::KEY_T, false)]);

    assert_eq!(layout.get_active_layers(), vec![0, 1]);

    // Time was too long for a tap key
    layout.process_keyevent(KeyStateChange::Released(TestDevice::B01), t.advance_ms(220));
    assert_emitted_keys(&mut layout, vec![]);

    assert_eq!(layout.get_active_layers(), vec![0]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B04), t);
    assert_emitted_keys(&mut layout, vec![]);
}

// Dual layout, basic test simulating hold layer with key timeout behavior
fn hold_and_tap_keygroup_layered_layout() -> LayerSwitcher {
    let keymap_default = vec![ // blocks
        vec![ // rows
            vec![ LhtKg(1, vec![Key::KEY_LEFTALT, Key::KEY_0]),   K(Key::KEY_B) ],
            vec![ K(Key::KEY_LEFTSHIFT),                          No,           ],
        ],
    ];

    let keymap_shift = vec![ // blocks
        vec![ // rows
            vec![ No,                    K(Key::KEY_T), ],
            vec![ K(Key::KEY_LEFTSHIFT), K(Key::KEY_E), ],
        ],
    ];

    let default_layer = Layer{
        keymap: keymap_default,
        ..DEFAULT_LAYER_CONFIG
    };

    let shift_layer = Layer{
        status_on_reset: crate::layout::types::LayerStatus::LayerPassthrough,
        keymap: keymap_shift,
        ..DEFAULT_LAYER_CONFIG
    };

    let layers = vec![default_layer, shift_layer];

    LayerSwitcher::new(layers)
}

#[test]
fn test_hold_and_tap_keygroup_layered_layout() {
    let mut layout = hold_and_tap_keygroup_layered_layout();
    layout.start();
    let mut t = TestTime::start();

    assert_emitted_keys(&mut layout, vec![]);

    layout.process_keyevent(KeyStateChange::Pressed(TestDevice::B01), t);
    assert_emitted_keys(&mut layout, vec![]);

    assert_eq!(layout.get_active_layers(), vec![0, 1]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B02), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_T, true), (Key::KEY_T, false)]);

    assert_eq!(layout.get_active_layers(), vec![0, 1]);

    // Time was short enough for tap key
    layout.process_keyevent(KeyStateChange::Released(TestDevice::B01), t.advance_ms(190));
    assert_emitted_keys(&mut layout, vec![(Key::KEY_LEFTALT, true), (Key::KEY_0, true), (Key::KEY_0, false), (Key::KEY_LEFTALT, false)]);

    assert_eq!(layout.get_active_layers(), vec![0]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B04), t);
    assert_emitted_keys(&mut layout, vec![]);
}

#[test]
fn test_hold_and_tap_keygroup_layered_layout_long_press() {
    let mut layout = hold_and_tap_keygroup_layered_layout();
    layout.start();
    let mut t = TestTime::start();

    assert_emitted_keys(&mut layout, vec![]);

    layout.process_keyevent(KeyStateChange::Pressed(TestDevice::B01), t);
    assert_emitted_keys(&mut layout, vec![]);

    assert_eq!(layout.get_active_layers(), vec![0, 1]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B02), t);
    assert_emitted_keys(&mut layout, vec![(Key::KEY_T, true), (Key::KEY_T, false)]);

    assert_eq!(layout.get_active_layers(), vec![0, 1]);

    // Time was too long for a tap key
    layout.process_keyevent(KeyStateChange::Released(TestDevice::B01), t.advance_ms(220));
    assert_emitted_keys(&mut layout, vec![]);

    assert_eq!(layout.get_active_layers(), vec![0]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B04), t);
    assert_emitted_keys(&mut layout, vec![]);
}