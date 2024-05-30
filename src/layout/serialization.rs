use std::collections::VecDeque;

use toml;
use evdev::Key;

use super::layer::Layer;
use super::switcher::LayerSwitcher;
use super::types::KeymapEvent::{Inh, No, Ldisable, Lactivate, Lhold, Lmove, Ltap, K, Ks, Kms, Pass, Kg};

/*

( CCW=10 ROT CW=11 ) [ 0 ][ 1 ][ 2 ][ 6 ]
                     [ 3 ][ 4 ][ 5 ][ _ ]
                     [ 7 ][    8   ][ 9 ]
 */


pub fn load_layout(s: &str) -> LayerSwitcher {
    let keymap_default = vec![ // blocks
        vec![ // rows
            vec![ K(Key::KEY_LEFTALT),   K(Key::KEY_B),                            K(Key::KEY_E),
                  K(Key::KEY_LEFTSHIFT), K(Key::KEY_INSERT),                       No,              K(Key::KEY_SLASH),
                  K(Key::KEY_LEFTCTRL),  Kg(vec![Key::KEY_LEFTCTRL, Key::KEY_E]),  No,

                  K(Key::KEY_ZOOMOUT),   K(Key::KEY_ZOOMIN) ]
        ],
    ];

    let default_layer = Layer{
        status_on_reset: super::types::LayerStatus::LayerActive,
        inherit: None,
        on_active_keys: vec![],
        on_timeout_layer: None,
        timeout: None,
        keymap: keymap_default,
        default_action: super::types::KeymapEvent::No,
    };

    let layers = vec![default_layer];

    LayerSwitcher::new(layers)
}