use evdev::Key;
use toml;

use super::keys::{G, S};
use super::layer::Layer;
use super::types::KeymapEvent::{
    Inh, Kg, Klong, Lactivate, Ldisable, Lhold, LhtK, Lmove, Ltap, No, Pass,
};

/*

( CCW=10 ROT CW=11 ) [ 0 ][ 1 ][ 2 ][ 6 ]
                     [ 3 ][ 4 ][ 5 ][ _ ]
                     [ 7 ][    8   ][ 9 ]

 or in the other orientation

 [ 9 ][    8   ][ 7 ]
 [ 6 |[ 5 ][ 4 ][ 3 ]
 | _ ][ 2 ][ 1 ][ 0 ]  ( CCW=10 ROT CW=11 )

 */

pub fn load_layout(s: &str) -> Vec<Layer> {
    // Layer 0 - default
    let keymap_default = vec![
        // blocks
        vec![
            // rows
            vec![
                /*  0  */
                No,
                /*  1  */
                No,
                /*  2  */
                Klong(G(), G().k(Key::KEY_DELETE)),
                /*  3  */
                Lhold(3),
                /*  4  */
                LhtK(1, G().k(Key::KEY_B)),
                /*  5  */
                LhtK(4, G()),
                /*  6  */
                G().k(Key::KEY_LEFTCTRL).k(Key::KEY_Z).p(),
                /*  7  */
                LhtK(5, G().k(Key::KEY_INSERT)),
                /*  8  */
                LhtK(2, G().k(Key::KEY_LEFTSHIFT).k(Key::KEY_E)),
                /*  9  */
                Klong(
                    G().k(Key::KEY_F12),
                    G().k(Key::KEY_LEFTCTRL).k(Key::KEY_LEFTSHIFT).k(Key::KEY_A),
                ),
                /* CCW */
                G().k(Key::KEY_MINUS).p(),
                /*  CW */
                G().k(Key::KEY_SLASH).p(), // should be minus and equals
            ],
        ],
    ];

    let default_layer = Layer {
        status_on_reset: super::types::LayerStatus::LayerActive,
        inherit: None,
        on_active_keys: vec![],
        disable_active_on_press: false,
        on_timeout_layer: None,
        timeout: None,
        keymap: keymap_default,
        default_action: super::types::KeymapEvent::Pass,
    };


    // Layer 1 - Color
    let keymap_color = vec![
        // blocks
        vec![
            // rows
            vec![
                /*  0  */
                No,
                /*  1  */
                G().k(Key::KEY_LEFTCTRL).k(Key::KEY_E).p(),
                /*  2  */
                Pass,
                /*  3  */
                G().k(Key::KEY_K).p(),
                /*  4  */
                No,
                /*  5  */
                No,
                /*  6  */
                G().k(Key::KEY_SLASH).p(),
                /*  7  */
                G().k(Key::KEY_L).p(),
                /*  8  */
                G().k(Key::KEY_LEFTCTRL).k(Key::KEY_SPACE).p(),
                /*  9  */
                No,
                /* CCW */
                G().k(Key::KEY_RIGHTBRACE).p(),
                /*  CW */
                G().k(Key::KEY_LEFTBRACE).p(),
            ],
        ],
    ];

    let color_layer = Layer {
        status_on_reset: super::types::LayerStatus::LayerPassthrough,
        on_active_keys: vec![Key::KEY_LEFTCTRL],
        disable_active_on_press: true,
        keymap: keymap_color,
        ..default_layer.clone()
    };


    // Layer 2 - Tools
    let keymap_tools = vec![
        // blocks
        vec![
            // rows
            vec![
                /*  0  */
                G().k(Key::KEY_ESC).p(),
                /*  1  */
                G().k(Key::KEY_5).p(),
                /*  2  */
                G().k(Key::KEY_LEFTCTRL).k(Key::KEY_T).p(),
                /*  3  */
                No,
                /*  4  */
                G().k(Key::KEY_ENTER).p(),
                /*  5  */
                No,
                /*  6  */
                No,
                /*  7  */
                G().k(Key::KEY_LEFTCTRL).k(Key::KEY_SPACE).p(),
                /*  8  */
                No,
                /*  9  */
                G().k(Key::KEY_T).p(),
                /* CCW */
                G().k(Key::KEY_6).p(),
                /*  CW */
                G().k(Key::KEY_4).p(),
            ],
        ],
    ];

    let tools_layer = Layer {
        status_on_reset: super::types::LayerStatus::LayerPassthrough,
        on_active_keys: vec![Key::KEY_LEFTSHIFT],
        disable_active_on_press: true,
        keymap: keymap_tools,
        ..default_layer.clone()
    };


    // Layer 3 - View
    let keymap_view = vec![
        // blocks
        vec![
            // rows
            vec![
                /*  0  */
                No,
                /*  1  */
                G().k(Key::KEY_4).p(),
                /*  2  */
                G().k(Key::KEY_6).p(),
                /*  3  */
                G().k(Key::KEY_LEFTCTRL)
                    .k(Key::KEY_LEFTSHIFT)
                    .k(Key::KEY_Z)
                    .p(),
                /*  4  */
                Pass,
                /*  5  */
                G().k(Key::KEY_5).p(),
                /*  6  */
                No,
                /*  7  */
                No,
                /*  8  */
                G().k(Key::KEY_LEFTCTRL).k(Key::KEY_SPACE).p(),
                /*  9  */
                No,
                /* CCW */
                Pass,
                /*  CW */
                Pass,
            ],
        ],
    ];

    let view_layer = Layer {
        status_on_reset: super::types::LayerStatus::LayerPassthrough,
        on_active_keys: vec![Key::KEY_SPACE],
        disable_active_on_press: true,
        keymap: keymap_view,
        ..default_layer.clone()
    };


    // Used in Layer 4 - Drawing
    let keymap_pass = vec![
        // blocks
        vec![
            // rows
            vec![
                /*  0  */ Pass, /*  1  */ Pass, /*  2  */ Pass, /*  3  */ Pass,
                /*  4  */ Pass, /*  5  */ Pass, /*  6  */ Pass, /*  7  */ Pass,
                /*  8  */ Pass, /*  9  */ Pass, /* CCW */ Pass, /*  CW */ Pass,
            ],
        ],
    ];

    let draw_layer = Layer {
        status_on_reset: super::types::LayerStatus::LayerPassthrough,
        on_active_keys: vec![Key::KEY_V],
        disable_active_on_press: true,
        keymap: keymap_pass,
        ..default_layer.clone()
    };

    // Layer 5 - Layer actions
    let keymap_layer = vec![
        // blocks
        vec![
            // rows
            vec![
                /*  0  */
                Pass,
                /*  1  */
                Pass,
                /*  2  */
                Pass,
                /*  3  */
                Pass,
                /*  4  */
                Pass,
                /*  5  */
                Pass,
                /*  6  */
                Pass,
                /*  7  */
                Pass,
                /*  8  */
                G().k(Key::KEY_LEFTCTRL).k(Key::KEY_E).p(),
                /*  9  */
                Pass,
                /* CCW */
                Pass,
                /*  CW */
                Pass,
            ],
        ],
    ];

    let layers_layer = Layer {
        status_on_reset: super::types::LayerStatus::LayerPassthrough,
        on_active_keys: vec![],
        disable_active_on_press: true,
        keymap: keymap_layer,
        ..default_layer.clone()
    };


    // Layer ordering, do not change!

    let layers = vec![
        default_layer,
        color_layer,
        tools_layer,
        view_layer,
        draw_layer,
        layers_layer,
    ];

    layers
}
