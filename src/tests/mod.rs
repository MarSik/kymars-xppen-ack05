use evdev::Key;

use crate::kbd_events::KeyStateChange;
use crate::layout::layer::Layer;
use crate::layout::types::KeyCoords;
use crate::layout::switcher::LayerSwitcher;
use crate::layout::types::KeymapEvent::{K, Kms, No, Lhold, Inh, Ltap, Lactivate, Pass};

#[non_exhaustive]
struct TestDevice;

impl TestDevice {
    pub(crate) const B01: KeyCoords = (0, 0, 0);
    pub(crate) const B02: KeyCoords = (0, 0, 1);
    pub(crate) const B03: KeyCoords = (0, 1, 0);
    pub(crate) const B04: KeyCoords = (0, 1, 1);
}

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
        status_on_reset: crate::layout::types::LayerStatus::LayerActive,
        inherit: None,
        on_active_keys: vec![],
        on_timeout_layer: None,
        timeout: None,
        keymap: keymap_default,
        default_action: crate::layout::types::KeymapEvent::No,
    };

    let layers = vec![default_layer];

    LayerSwitcher::new(layers)
}

#[test]
fn test_basic_layout() {
    let mut layout = basic_layout();
    layout.start();
    assert_emitted_keys(&mut layout, vec![]);

    layout.process_keyevent(KeyStateChange::Pressed(TestDevice::B01));
    assert_emitted_keys(&mut layout, vec![(Key::KEY_LEFTALT, true)]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B02));
    assert_emitted_keys(&mut layout, vec![(Key::KEY_B, true), (Key::KEY_B, false)]);

    layout.process_keyevent(KeyStateChange::Released(TestDevice::B01));
    assert_emitted_keys(&mut layout, vec![(Key::KEY_LEFTALT, false)]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B04));
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
        status_on_reset: crate::layout::types::LayerStatus::LayerActive,
        inherit: None,
        on_active_keys: vec![],
        on_timeout_layer: None,
        timeout: None,
        keymap: keymap_default,
        default_action: crate::layout::types::KeymapEvent::No,
    };

    let shift_layer = Layer{
        status_on_reset: crate::layout::types::LayerStatus::LayerPassthrough,
        inherit: Some(2),
        on_active_keys: vec![Key::KEY_LEFTSHIFT],
        on_timeout_layer: None,
        timeout: None,
        keymap: keymap_shift,
        default_action: crate::layout::types::KeymapEvent::No,
    };

    let inh_layer = Layer{
        status_on_reset: crate::layout::types::LayerStatus::LayerDisabled,
        inherit: None,
        on_active_keys: vec![],
        on_timeout_layer: None,
        timeout: None,
        keymap: keymap_inh,
        default_action: crate::layout::types::KeymapEvent::No,
    };

    let layers = vec![default_layer, shift_layer, inh_layer];

    LayerSwitcher::new(layers)
}

#[test]
fn test_basic_layered_layout() {
    let mut layout = basic_layered_layout();
    layout.start();
    assert_emitted_keys(&mut layout, vec![]);

    layout.process_keyevent(KeyStateChange::Pressed(TestDevice::B01));
    assert_emitted_keys(&mut layout, vec![(Key::KEY_LEFTSHIFT, true)]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B02));
    assert_emitted_keys(&mut layout, vec![(Key::KEY_B, true), (Key::KEY_B, false)]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B04));
    assert_emitted_keys(&mut layout, vec![(Key::KEY_E, true), (Key::KEY_E, false)]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B03));
    assert_emitted_keys(&mut layout, vec![(Key::KEY_2, true), (Key::KEY_2, false)]);

    layout.process_keyevent(KeyStateChange::Released(TestDevice::B01));
    assert_emitted_keys(&mut layout, vec![(Key::KEY_LEFTSHIFT, false)]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B04));
    assert_emitted_keys(&mut layout, vec![]);
}

#[test]
fn test_basic_layered_layout_cross_release() {
    let mut layout = basic_layered_layout();
    layout.start();
    assert_emitted_keys(&mut layout, vec![]);

    layout.process_keyevent(KeyStateChange::Pressed(TestDevice::B01));
    assert_emitted_keys(&mut layout, vec![(Key::KEY_LEFTSHIFT, true)]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B02));
    assert_emitted_keys(&mut layout, vec![(Key::KEY_B, true), (Key::KEY_B, false)]);

    layout.process_keyevent(KeyStateChange::Pressed(TestDevice::B04));
    assert_emitted_keys(&mut layout, vec![(Key::KEY_E, true),]);

    // TODO how should this behave? Should KEY_E really be released together with the shift
    //      when the layer deactivates?
    layout.process_keyevent(KeyStateChange::Released(TestDevice::B01));
    assert_emitted_keys(&mut layout, vec![(Key::KEY_LEFTSHIFT, false), (Key::KEY_E, false)]);

    layout.process_keyevent(KeyStateChange::Released(TestDevice::B04));
    assert_emitted_keys(&mut layout, vec![]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B04));
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
        status_on_reset: crate::layout::types::LayerStatus::LayerActive,
        inherit: None,
        on_active_keys: vec![],
        on_timeout_layer: None,
        timeout: None,
        keymap: keymap_default,
        default_action: crate::layout::types::KeymapEvent::No,
    };

    let shift_layer = Layer{
        status_on_reset: crate::layout::types::LayerStatus::LayerPassthrough,
        inherit: None,
        on_active_keys: vec![Key::KEY_LEFTSHIFT],
        on_timeout_layer: None,
        timeout: None,
        keymap: keymap_shift,
        default_action: crate::layout::types::KeymapEvent::No,
    };

    let layers = vec![default_layer, shift_layer];

    LayerSwitcher::new(layers)
}

#[test]
fn test_tap_layered_layout() {
    let mut layout = tap_layered_layout();
    layout.start();
    assert_emitted_keys(&mut layout, vec![]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B01));
    assert_emitted_keys(&mut layout, vec![(Key::KEY_LEFTSHIFT, true)]);

    assert_eq!(layout.get_active_layers(), vec![0, 1]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B02));
    assert_emitted_keys(&mut layout, vec![(Key::KEY_B, true), (Key::KEY_LEFTSHIFT, false), (Key::KEY_B, false)]);

    assert_eq!(layout.get_active_layers(), vec![0]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B04));
    assert_emitted_keys(&mut layout, vec![]);

    layout.process_keyevent(KeyStateChange::Released(TestDevice::B01));
    assert_emitted_keys(&mut layout, vec![]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B04));
    assert_emitted_keys(&mut layout, vec![]);
}

// Dual layout, basic test simulating Shift behavior (hold to stay in the second layer),
// but with a key in second layer disabling shift temporarily
fn masked_layered_layout() -> LayerSwitcher {
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
        status_on_reset: crate::layout::types::LayerStatus::LayerActive,
        inherit: None,
        on_active_keys: vec![],
        on_timeout_layer: None,
        timeout: None,
        keymap: keymap_default,
        default_action: crate::layout::types::KeymapEvent::No,
    };

    let shift_layer = Layer{
        status_on_reset: crate::layout::types::LayerStatus::LayerPassthrough,
        inherit: None,
        on_active_keys: vec![Key::KEY_LEFTSHIFT],
        on_timeout_layer: None,
        timeout: None,
        keymap: keymap_shift,
        default_action: crate::layout::types::KeymapEvent::No,
    };

    let layers = vec![default_layer, shift_layer];

    LayerSwitcher::new(layers)
}

#[test]
fn test_masked_layered_layout() {
    let mut layout = masked_layered_layout();
    layout.start();
    assert_emitted_keys(&mut layout, vec![]);

    layout.process_keyevent(KeyStateChange::Pressed(TestDevice::B01));
    assert_emitted_keys(&mut layout, vec![(Key::KEY_LEFTSHIFT, true)]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B02));
    assert_emitted_keys(&mut layout, vec![(Key::KEY_B, true), (Key::KEY_B, false)]);

    // This temporarily masks the Shift key
    layout.process_keyevent(KeyStateChange::Click(TestDevice::B04));
    assert_emitted_keys(&mut layout, vec![(Key::KEY_LEFTSHIFT, false), (Key::KEY_E, true), (Key::KEY_E, false), (Key::KEY_LEFTSHIFT, true)]);

    layout.process_keyevent(KeyStateChange::Released(TestDevice::B01));
    assert_emitted_keys(&mut layout, vec![(Key::KEY_LEFTSHIFT, false)]);

    layout.process_keyevent(KeyStateChange::Click(TestDevice::B04));
    assert_emitted_keys(&mut layout, vec![]);
}
