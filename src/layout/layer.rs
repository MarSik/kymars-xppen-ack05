use std::time::Duration;

use evdev::Key;

use super::types::{KeyCoords, Keymap, KeymapEvent, LayerId, LayerStatus};

#[derive(Clone)]
pub struct Layer {
    // Should be active on reset?
    pub(crate) status_on_reset: LayerStatus,

    // Where to inherit from when KeymapEvent.Inh is used
    pub(crate) inherit: Option<LayerId>,

    // A key event to send when this layer is active
    pub(crate) on_active_keys: Vec<Key>,

    // A layer switch when timer expires
    pub(crate) on_timeout_layer: Option<LayerId>,

    // Timeout to setup when layer is entered
    pub(crate) timeout: Option<Duration>,

    // Keymap definition when this layer is active
    pub(crate) keymap: Keymap,

    pub(crate) default_action: KeymapEvent,
}

impl Layer {
    pub fn get_key_event(&self, coords: KeyCoords) -> KeymapEvent {
        self.keymap.get(coords.0 as usize)
            .and_then(|block| block.get(coords.1 as usize))
            .and_then(|row| row.get(coords.2 as usize))
            .unwrap_or(&self.default_action)
            .clone()
    }

    pub fn get_used_keys(&self) -> Vec<Key> {
        let mut keys = Vec::new();
        for b in &self.keymap {
            for r in b {
                for ev in r {
                    match ev {
                        KeymapEvent::No => {},
                        KeymapEvent::Inh => {},
                        KeymapEvent::Pass => {},
                        KeymapEvent::K(k) => keys.push(*k),
                        KeymapEvent::Kg(ks) => {
                            keys.extend(ks);
                        },
                        KeymapEvent::Ks(ks) => keys.extend(ks),
                        KeymapEvent::Kms(km, kc) => {
                            keys.extend(km);
                            keys.extend(kc);
                        },
                        _ => {}
                    }
                }
            }
        }
        return keys;
    }
}