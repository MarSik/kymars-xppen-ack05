use std::time::Instant;

use super::keys::KeyGroup;

pub type LayerId = usize;
pub type EventCount = u32;

#[derive(Clone, Copy, PartialEq)]
pub enum LayerStatus {
    /// Layer active. Can only be deactivated explicitly.
    LayerActive,
    /// Layer inactive, does not participate in key resolution.
    LayerPassthrough,
    /// Layer active while the key is held down.
    LayerActiveUntilKeyRelease(KeyCoords),
    /// Layer active while the key is held down and until one additional
    /// keypress happens after the key is released.
    LayerActiveUntilKeyReleaseTap(KeyCoords),
    /// Layer active for one additional keypress.
    LayerActiveUntilAnyKeyPress,
    /// Layer active while the activation key is being held down. On release this
    /// can trigger another layer activation if the duration of the press was short.
    LayerHoldAndTapToL(KeyCoords, Instant, LayerId),
    /// Layer active while the recorded key is being held down. On release this
    /// can trigger key group press and release if the duration of the press was short.
    LayerHoldAndTapKey(KeyCoords, Instant, LayerId), // The key action is retrieved from the keymap
    /// Layer unconditionally disabled, does not participate in key resolution
    /// And can only be enabled explicitly
    LayerDisabled,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyCoords(pub u8, pub u8, pub u8); // Block, row, column

pub type Keymap = Vec<Vec<Vec<KeymapEvent>>>; // [Block, Row, Col] - > default KeyEvent(None)

#[derive(Clone, PartialEq)]
pub enum KeymapEvent {
    /// No effect, no inheritance
    No,
    /// Inherit effect from layer.inherit layer
    Inh,
    /// No effect, check other active layers next
    Pass,
    /// Map key press/release to a keycode
    Kg(KeyGroup),
    /// If a key is released quickly send first key press/release pair,
    /// but when it is still pressed after the timeout, press the second key
    /// and release it on key release.
    Klong(KeyGroup, KeyGroup),
    /// A short press for key, long press for activating a layer
    Khl(KeyGroup, LayerId),
    /// A short press for key, long press for activating a tap layer (Ltap)
    Khtl(KeyGroup, LayerId),

    /// Disable all layers except the base and the parameter
    Lmove(LayerId),
    /// Activate a layer
    Lactivate(LayerId),
    /// Deactivate a layer
    Ldeactivate(LayerId),
    /// Permanently disable a layer
    Ldisable(LayerId),
    /// Activate layer while the initiating key is kept pressed. Deactivate on release.
    Lhold(LayerId),
    /// Activate layer while the initiating key is kept pressed. Deactivate after one additional key
    /// is pressed when the activating key is already releases. (Dead key behavior)
    Ltap(LayerId),
    /// Activate the first mentioned layer on press and deactivate on release. Additionally,
    /// if the elapsed time between press and release was short, activate the second layer.
    LhtL(LayerId, LayerId),
    /// Activate the first mentioned layer on press and deactivate on release. Additionally,
    /// if the elapsed time between press and release was short, send a press+release key event.
    LhtK(LayerId, KeyGroup),
}
