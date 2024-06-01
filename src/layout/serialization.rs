use std::collections::VecDeque;

use toml;
use evdev::Key;

use super::layer::Layer;
use super::switcher::LayerSwitcher;
use super::types::KeymapEvent::{Inh, No, Ldisable, Lactivate, Lhold, Lmove, Ltap, K, Ks, Kms, Pass, Kg, LhtK, LhtKg};

/*

( CCW=10 ROT CW=11 ) [ 0 ][ 1 ][ 2 ][ 6 ]
                     [ 3 ][ 4 ][ 5 ][ _ ]
                     [ 7 ][    8   ][ 9 ]
 */


pub fn load_layout(s: &str) -> LayerSwitcher {
    let keymap_default = vec![ // blocks
        vec![ // rows
            vec![ K(Key::KEY_F12), K(Key::KEY_INSERT),                             Kg(vec![Key::KEY_LEFTSHIFT, Key::KEY_E]),
                  No,            No,                                             LhtK(4, Key::KEY_B),                       Kg(vec![Key::KEY_LEFTCTRL, Key::KEY_Z]),
                  Lhold(1),        LhtKg(2, vec![Key::KEY_LEFTSHIFT, Key::KEY_E]),                                            Lhold(3),

                  K(Key::KEY_MINUS), K(Key::KEY_SLASH) ] // should be minus and equals
        ],
    ];

    let keymap_color = vec![ // blocks
        vec![ // rows
            vec![ K(Key::KEY_L),         Kg(vec![Key::KEY_LEFTCTRL, Key::KEY_E]),      Pass,
                  K(Key::KEY_K),         No,                                           No,              K(Key::KEY_SLASH),
                  No,                    Kg(vec![Key::KEY_LEFTCTRL, Key::KEY_SPACE]),  No,

                  K(Key::KEY_RIGHTBRACE),   K(Key::KEY_LEFTBRACE) ]
        ],
    ];

    let keymap_view = vec![ // blocks
    vec![ // rows
        vec![ No,             K(Key::KEY_4),     K(Key::KEY_6),
              No,   Pass,     K(Key::KEY_5),     Kg(vec![Key::KEY_LEFTCTRL, Key::KEY_LEFTSHIFT, Key::KEY_Z]),
              No,             Pass,     No,

              Pass, Pass   ]
        ],
    ];

    let keymap_tools = vec![ // blocks
    vec![ // rows
        vec![ K(Key::KEY_ESC),                               Kg(vec![Key::KEY_LEFTCTRL, Key::KEY_E]),   Kg(vec![Key::KEY_LEFTCTRL, Key::KEY_T]),
              No,                                            K(Key::KEY_5),                             No,        K(Key::KEY_ENTER),
              Kg(vec![Key::KEY_LEFTCTRL, Key::KEY_SPACE]),   No,                                        Kg(vec![Key::KEY_LEFTSHIFT, Key::KEY_SPACE]),

              K(Key::KEY_6), K(Key::KEY_4) ]
        ],
    ];

    let keymap_pass = vec![ // blocks
    vec![ // rows
        vec![ Pass,   Pass,   Pass,
              Pass,   Pass,   Pass,  Pass,
              Pass,   Pass,   Pass,

              Pass, Pass ]
        ],
    ];

    let default_layer = Layer{
        status_on_reset: super::types::LayerStatus::LayerActive,
        inherit: None,
        on_active_keys: vec![],
        disable_active_on_press: false,
        on_timeout_layer: None,
        timeout: None,
        keymap: keymap_default,
        default_action: super::types::KeymapEvent::Pass,
    };

    let color_layer = Layer{
        status_on_reset: super::types::LayerStatus::LayerPassthrough,
        on_active_keys: vec![Key::KEY_LEFTCTRL],
        disable_active_on_press: true,
        keymap: keymap_color,
        ..default_layer.clone()
    };

    let tools_layer = Layer{
        status_on_reset: super::types::LayerStatus::LayerPassthrough,
        on_active_keys: vec![Key::KEY_LEFTSHIFT],
        disable_active_on_press: true,
        keymap: keymap_tools,
        ..default_layer.clone()
    };

    let view_layer = Layer{
        status_on_reset: super::types::LayerStatus::LayerPassthrough,
        on_active_keys: vec![Key::KEY_SPACE],
        disable_active_on_press: true,
        keymap: keymap_view,
        ..default_layer.clone()
    };

    let draw_layer = Layer{
        status_on_reset: super::types::LayerStatus::LayerPassthrough,
        on_active_keys: vec![Key::KEY_V],
        disable_active_on_press: true,
        keymap: keymap_pass,
        ..default_layer.clone()
    };

    let layers = vec![default_layer, color_layer, tools_layer, view_layer, draw_layer];

    LayerSwitcher::new(layers)
}