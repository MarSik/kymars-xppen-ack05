use std::collections::VecDeque;

use toml;
use evdev::Key;

use super::layer::Layer;
use super::switcher::LayerSwitcher;
use super::types::KeymapEvent::{Inh, No, Ldisable, Lactivate, Lhold, Lmove, Ltap, Kg, Pass, LhtK};
use super::keys::{G, S};

/*

( CCW=10 ROT CW=11 ) [ 0 ][ 1 ][ 2 ][ 6 ]
                     [ 3 ][ 4 ][ 5 ][ _ ]
                     [ 7 ][    8   ][ 9 ]
 */


pub fn load_layout(s: &str) -> Vec<Layer> {
    let keymap_default = vec![ // blocks
        vec![ // rows
            vec![ G().k(Key::KEY_F12).p(), G().k(Key::KEY_INSERT).p(),                             G().k(Key::KEY_LEFTSHIFT).k(Key::KEY_E).p(),
                  No,            No,                                             LhtK(4, G().k(Key::KEY_B)),                       G().k(Key::KEY_LEFTCTRL).k(Key::KEY_Z).p(),
                  Lhold(1),        LhtK(2, G().k(Key::KEY_LEFTSHIFT).k(Key::KEY_E)),                                            Lhold(3),

                  G().k(Key::KEY_MINUS).p(), G().k(Key::KEY_SLASH).p() ] // should be minus and equals
        ],
    ];

    let keymap_color = vec![ // blocks
        vec![ // rows
            vec![ G().k(Key::KEY_L).p(),         G().k(Key::KEY_LEFTCTRL).k(Key::KEY_E).p(),      Pass,
                  G().k(Key::KEY_K).p(),         No,                                           No,              G().k(Key::KEY_SLASH).p(),
                  No,                    G().k(Key::KEY_LEFTCTRL).k(Key::KEY_SPACE).p(),  No,

                  G().k(Key::KEY_RIGHTBRACE).p(),   G().k(Key::KEY_LEFTBRACE).p() ]
        ],
    ];

    let keymap_view = vec![ // blocks
    vec![ // rows
        vec![ No,             G().k(Key::KEY_4).p(),     G().k(Key::KEY_6).p(),
              No,   Pass,     G().k(Key::KEY_5).p(),     G().k(Key::KEY_LEFTCTRL).k(Key::KEY_LEFTSHIFT).k(Key::KEY_Z).p(),
              No,             Pass,     No,

              Pass, Pass   ]
        ],
    ];

    let keymap_tools = vec![ // blocks
    vec![ // rows
        vec![ G().k(Key::KEY_ESC).p(),                               G().k(Key::KEY_LEFTCTRL).k(Key::KEY_E).p(),   G().k(Key::KEY_LEFTCTRL).k(Key::KEY_T).p(),
              No,                                            G().k(Key::KEY_5).p(),                             No,        G().k(Key::KEY_ENTER).p(),
              G().k(Key::KEY_LEFTCTRL).k(Key::KEY_SPACE).p(),   No,                                        G().k(Key::KEY_LEFTSHIFT).k(Key::KEY_SPACE).p(),

              G().k(Key::KEY_6).p(), G().k(Key::KEY_4).p() ]
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

    layers
}