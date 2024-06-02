use std::time::Instant;

use evdev::Key;

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

#[derive(Clone)]
pub enum KeymapEvent {
    /// No effect, no inheritance
    No,
    /// Inherit effect from layer.inherit layer
    Inh,
    /// No effect, check other active layers next
    Pass,
    /// Map key press/release to a keycode
    K(evdev::Key),
    /// Press event sends a sequence of key press/release events.
    /// Release does nothing.
    Ks(Vec<evdev::Key>), // Key click sequence
    /// Press event sends a sequence of key press events.
    /// Release sends the a key release sequence in reverse order
    Kg(Vec<evdev::Key>), // Key group
    /// Key event with mask. First a key release event is sent for each mask key,
    /// then a click (press followed by release) of keys and at the end the mask
    /// is replayed as keypress events in reverse order (the same as Kg)
    Kms(Vec<evdev::Key>, Vec<evdev::Key>), // mask, click sequence
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
    LhtK(LayerId, Key),
    /// Activate the first mentioned layer on press and deactivate on release. Additionally,
    /// if the elapsed time between press and release was short, send a sequence of key press
    /// events followed by a reverse sequence of key release events (the same as Kg)
    LhtKg(LayerId, Vec<Key>),
}
