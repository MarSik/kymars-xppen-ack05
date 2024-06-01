use std::time::Instant;

use evdev::Key;

pub type LayerId = usize;
pub type EventCount = u32;

#[derive(Clone, Copy, PartialEq)]
pub enum LayerStatus {
    LayerActive,
    LayerPassthrough,
    LayerActiveUntilKeyRelease(KeyCoords),
    LayerActiveUntilKeyReleaseTap(KeyCoords),
    LayerActiveUntilAnyKeyPress,
    LayerHoldAndTapToL(KeyCoords, Instant, LayerId),
    LayerHoldAndTapKey(KeyCoords, Instant, LayerId), // The key action is retrieved from the keymap
    LayerDisabled,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyCoords(pub u8, pub u8, pub u8); // Block, row, column

pub type Keymap = Vec<Vec<Vec<KeymapEvent>>>; // [Block, Row, Col] - > default KeyEvent(None)

#[derive(Clone)]
pub enum KeymapEvent {
    No,
    Inh,
    Pass,
    K(evdev::Key),
    Ks(Vec<evdev::Key>), // Key click sequence
    Kg(Vec<evdev::Key>), // Key group
    Kms(Vec<evdev::Key>, Vec<evdev::Key>), // mask, click sequence
    Lmove(LayerId),
    Lactivate(LayerId),
    Ldeactivate(LayerId),
    Ldisable(LayerId),
    Lhold(LayerId),
    Ltap(LayerId),
    LhtL(LayerId, LayerId),
    LhtK(LayerId, Key),
    LhtKg(LayerId, Vec<Key>),
}
