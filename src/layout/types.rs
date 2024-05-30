use std::{collections::{BTreeMap, BTreeSet}, time::Duration};

use evdev::Key;

use super::layer::Layer;

pub type LayerId = u16;
pub type EventCount = u32;

#[derive(Clone, Copy, PartialEq)]
pub enum LayerStatus {
    LayerActive,
    LayerPassthrough,
    LayerActiveUntilKeyRelease(KeyCoords),
    LayerActiveUntilAnyKeyReleaseBut(KeyCoords),
    LayerDisabled,
}

pub type KeyCoords = (u8, u8, u8); // Block, row, column

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
}
