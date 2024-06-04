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

    // Are active keys disabled when a key is pressed when the layer is active?
    pub(crate) disable_active_on_press: bool,

    // A layer switch when timer expires
    pub(crate) on_timeout_layer: Option<LayerId>,

    // Timeout to setup when layer is entered
    pub(crate) timeout: Option<Duration>,

    // Keymap definition when this layer is active
    pub(crate) keymap: Keymap,

    pub(crate) default_action: KeymapEvent,
}

impl Layer {
    pub fn get_key_event(&self, coords: KeyCoords) -> &KeymapEvent {
        self.keymap.get(coords.0 as usize)
            .and_then(|block| block.get(coords.1 as usize))
            .and_then(|row| row.get(coords.2 as usize))
            .unwrap_or(&self.default_action)
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
                        KeymapEvent::Kg(k) => keys.extend(k.get_used_keys()),
                        KeymapEvent::Klong(k_s, k_l) => {
                            keys.extend(k_s.get_used_keys());
                            keys.extend(k_l.get_used_keys());
                        },
                        KeymapEvent::Khtl(k, _) => keys.extend(k.get_used_keys()),
                        KeymapEvent::Khl(k, _) => keys.extend(k.get_used_keys()),

                        KeymapEvent::LhtK(_, k) => keys.extend(k.get_used_keys()),
                        _ => {}
                    }
                }
            }
        }
        return keys;
    }
}