use std::collections::{BTreeSet, HashSet, VecDeque};

use evdev::Key;

use crate::kbd_events::KeyStateChange;

use super::layer::Layer;
use super::types::{KeyCoords, KeymapEvent, LayerId, LayerStatus};

pub struct LayerSwitcher {
    pub(super) layers: Vec<Layer>,
    pub(super) layer_stack: Vec<LayerStackEntry>, // Each layer push adds an entry
    pub(super) emitted_codes: VecDeque<(evdev::Key, bool)>,

    // Cache of coordinates of all keys that emitted unclosed pressed key events
    // This is used to send key released events when a layer is deactivated and
    // the mapping to the originating K(key) is lost before the key is released.
    active_keys: BTreeSet<KeyCoords>,
}

#[derive(Clone)]
pub struct LayerStackEntry {
    pub(super) status: LayerStatus,
}

impl LayerSwitcher {
    pub fn new(layers: Vec<Layer>) -> Self {
        Self {
            layers,
            layer_stack: Vec::new(),
            emitted_codes: VecDeque::new(),
            active_keys: BTreeSet::new(),
        }
    }

    pub fn start(&mut self) {
        self.layer_stack.clear();
        for layer in &self.layers {
            self.layer_stack.push(LayerStackEntry { status: layer.status_on_reset })
        }
        self.layer_stack[0].status = LayerStatus::LayerActive;
        self.emitted_codes.clear();
        self.active_keys.clear();
    }

    fn layer_disable(&mut self, idx: usize) {
        // The lowest layer is always active
        if idx == 0 {
            return;
        }

        // Disabled layer, ignore action
        if self.layer_stack[idx].status == LayerStatus::LayerDisabled {
            return;
        }

        // Active layer, deactivate first
        if self.layer_stack[idx].status != LayerStatus::LayerPassthrough {
            self.layer_deactivate(idx);
        }

        self.layer_stack[idx].status = LayerStatus::LayerDisabled;
    }

    fn layer_deactivate(&mut self, idx: usize) {
        // The lowest layer is always active
        if idx == 0 {
            return;
        }

        // Disabled layer, ignore action
        if self.layer_stack[idx].status == LayerStatus::LayerDisabled {
            return;
        }

        // Non-active layer, ignore action
        if self.layer_stack[idx].status == LayerStatus::LayerPassthrough {
            return;
        }

        self.layer_stack[idx].status = LayerStatus::LayerPassthrough;

        self.on_layer_deactivation(idx);
    }

    fn layer_activate(&mut self, idx: usize) {
        // Disabled layer, ignore action
        if self.layer_stack[idx].status == LayerStatus::LayerDisabled {
            return;
        }

        // Active layer, ignore
        if self.layer_stack[idx].status == LayerStatus::LayerActive {
            return;
        }

        self.layer_stack[idx].status = LayerStatus::LayerActive;
        self.on_layer_activation(idx);
    }

    fn layer_hold(&mut self, idx: usize, coords: KeyCoords) {
        // Disabled layer, ignore action
        if self.layer_stack[idx].status == LayerStatus::LayerDisabled {
            return;
        }

        // Active layer, ignore action
        if self.layer_stack[idx].status != LayerStatus::LayerPassthrough {
            return;
        }

        self.layer_stack[idx].status = LayerStatus::LayerActiveUntilKeyRelease(coords);
        self.on_layer_activation(idx);
    }

    fn layer_tap(&mut self, idx: usize, coords: KeyCoords) {
        // Disabled layer, ignore action
        if self.layer_stack[idx].status == LayerStatus::LayerDisabled {
            return;
        }

        // Active layer, ignore action
        if self.layer_stack[idx].status != LayerStatus::LayerPassthrough {
            return;
        }

        self.layer_stack[idx].status = LayerStatus::LayerActiveUntilAnyKeyReleaseBut(coords);
        self.on_layer_activation(idx);
    }

    fn layer_move(&mut self, idx: usize) {
        // Disabled layer, ignore action
        if self.layer_stack[idx].status == LayerStatus::LayerDisabled {
            return;
        }

        // Compute all the active keys before layers are switched off
        let before = self.before_layer_deactivation();

        for (l_idx, l) in self.layer_stack.clone().into_iter().enumerate() {
            if idx == l_idx {
                continue;
            }
            self.layer_deactivate(idx);
        }

        self.layer_activate(idx);

        // Emit key released for all the "lost" keys
        self.after_layer_deactivation(before);
    }

    fn on_layer_activation(&mut self, idx: usize) {
        let keys = self.layers[idx].on_active_keys.clone();
        for k in keys {
            self.emit_keycodes((255, 255, 255), &k.clone(), true);
        }
    }

    fn on_layer_deactivation(&mut self, idx: usize) {
        let keys = self.layers[idx].on_active_keys.clone();
        for k in keys {
            self.emit_keycodes((255, 255, 255), &k.clone(), false);
        }
    }

    // Uses the internal cached list of pressed keys to compute all emitted keypresses
    fn before_layer_deactivation(&mut self) -> BTreeSet<Key> {
        let mut active_keys = BTreeSet::new();
        for coords in &self.active_keys {
            if let KeymapEvent::K(k) = self.get_key_event(*coords) {
                active_keys.insert(k);
            }
            if let KeymapEvent::Kg(ks) = self.get_key_event(*coords) {
                active_keys.extend(ks);
            }
        }
        return active_keys;
    }

    // Computes all emitted keypresses again and compares it to the set from
    // `before_layer_deactivation`. All lost keys emit key release events.
    fn after_layer_deactivation(&mut self, mut active_keys: BTreeSet<Key>) {
        // Compute all the active keys after layer deactivation
        for coords in &self.active_keys {
            if let KeymapEvent::K(k) = self.get_key_event(*coords) {
                active_keys.remove(&k);
            }
            if let KeymapEvent::Kg(ks) = self.get_key_event(*coords) {
                for k in ks {
                    active_keys.remove(&k);
                }
            }
        }

        // Emit key released for all the "lost" keys
        for k in active_keys {
            self.emit_keycodes((255, 255, 255), &k, false);
        }
    }

    fn process_keyevent_raw(&mut self, coords: KeyCoords, pressed: bool) {
        // Change layer states on key change (first deactivate on release)
        if !pressed {
            // Compute all the active keys before layers are switched off
            let mut active_keys = self.before_layer_deactivation();

            // Deactivate layers
            for (idx, l) in self.layer_stack.clone().into_iter().enumerate() {
                if let LayerStatus::LayerActiveUntilKeyRelease(wait_coords) = l.status {
                    if wait_coords == coords {
                        self.layer_deactivate(idx);
                    }
                } else if let LayerStatus::LayerActiveUntilAnyKeyReleaseBut(avoid_coords) = l.status {
                    if avoid_coords != coords {
                        self.layer_deactivate(idx);
                    }
                }
            }

            // Emit key released for all the "lost" keys
            self.after_layer_deactivation(active_keys);
        }

        // Identify the action associated with the current event
        let ev = self.get_key_event(coords);

        // Process the event
        match ev {
            // Nothing or indirection leading nowhere
            KeymapEvent::No => {},
            KeymapEvent::Inh => {},
            KeymapEvent::Pass => {},

            KeymapEvent::K(k) => {
                self.emit_keycodes(coords, &k, pressed);

                // Keep track of pressed keys in case of layer deactivation
                if pressed {
                    self.active_keys.insert(coords);
                } else {
                    self.active_keys.remove(&coords);
                }
            },
            KeymapEvent::Kg(ks) => {
                if pressed {
                    // Press group
                    for k in ks {
                        self.emit_keycodes(coords, &k, pressed);
                    }
                    // Keep track of pressed keys in case of layer deactivation
                    self.active_keys.insert(coords);

                } else {
                    // Release in reverse order
                    for k in ks.into_iter().rev() {
                        self.emit_keycodes(coords, &k, pressed);
                    }
                    self.active_keys.remove(&coords);
                };
            },
            KeymapEvent::Ks(ks) if pressed => {
                for k in ks {
                    self.emit_keycodes(coords, &k, true);
                    self.emit_keycodes(coords, &k, false);
                }
            },
            KeymapEvent::Kms(km, kc) if pressed => {
                for k in &km {
                    self.emit_keycodes(coords, &k, false);
                }
                for k in kc {
                    self.emit_keycodes(coords, &k, true);
                    self.emit_keycodes(coords, &k, false);
                }
                for k in &km {
                    self.emit_keycodes(coords, &k, true);
                }
            },
            KeymapEvent::Lmove(idx) if pressed => self.layer_move(idx as usize),
            KeymapEvent::Lhold(idx) if pressed => self.layer_hold(idx as usize, coords),
            KeymapEvent::Ltap(idx) if pressed => self.layer_tap(idx as usize, coords),
            KeymapEvent::Lactivate(idx) if pressed => self.layer_activate(idx as usize),

            KeymapEvent::Ldisable(idx) if pressed => {
                let mut active_keys = self.before_layer_deactivation();
                self.layer_disable(idx as usize);
                self.after_layer_deactivation(active_keys);

            },
            KeymapEvent::Ldeactivate(idx) if pressed => {
                let mut active_keys = self.before_layer_deactivation();
                self.layer_deactivate(idx as usize);
                self.after_layer_deactivation(active_keys);
            },
            _ => {}
        }
    }

    fn get_key_event(&self, coords: KeyCoords) -> KeymapEvent {
        'layer: for (idx, l) in (&self.layer_stack).into_iter().enumerate().rev() {
            // Skip disabled layers
            if l.status == LayerStatus::LayerDisabled || l.status == LayerStatus::LayerPassthrough {
                continue;
            }

            let mut layer_idx = idx;
            loop {
                let ev = (&self.layers)[layer_idx].get_key_event(coords);
                match ev {
                    KeymapEvent::No => return ev,
                    KeymapEvent::K(_) => return ev,
                    KeymapEvent::Kg(..) => return ev,
                    KeymapEvent::Ks(_) => return ev,
                    KeymapEvent::Kms(..) => return ev,
                    KeymapEvent::Lmove(_) => return ev,
                    KeymapEvent::Lhold(_) => return ev,
                    KeymapEvent::Ltap(_) => return ev,
                    KeymapEvent::Lactivate(_) => return ev,
                    KeymapEvent::Ldeactivate(_) => return ev,
                    KeymapEvent::Ldisable(_) => return ev,
                    KeymapEvent::Inh => {
                        // find the layer this inherits from
                        if let Some(next_p_idx) = (&self.layers)[layer_idx].inherit {
                            // TODO check that the parent layer ID is valid
                            layer_idx = next_p_idx as usize;
                        } else {
                            break // no parent
                        }
                    },
                    KeymapEvent::Pass => continue 'layer,
                }
            }
        }

        KeymapEvent::No
    }

    fn emit_keycodes(&mut self, coords: KeyCoords, k: &evdev::Key, pressed: bool) {
        self.emitted_codes.push_back((*k, pressed));
    }

    pub fn process_keyevent<T>(&mut self, ev: KeyStateChange<T>)
    where T: Into<KeyCoords>
    {
        assert!(self.layer_stack.len() > 0, "The layout engine was not started.");
        match ev {
            KeyStateChange::Pressed(k) => self.process_keyevent_raw(k.into(), true),
            KeyStateChange::Released(k) => self.process_keyevent_raw(k.into(), false),
            KeyStateChange::Click(k) => {
                let k = k.into();
                self.process_keyevent_raw(k, true);
                self.process_keyevent_raw(k, false);
            },
        }
    }

    pub fn render<F>(&mut self, mut renderer: F)
    where F: FnMut(Key, bool)
    {
        while let Some(k) = self.emitted_codes.pop_front() {
            renderer(k.0, k.1)
        }
    }

    pub fn get_used_keys(&self) -> HashSet<Key> {
        let mut keyset = HashSet::new();
        for l in &self.layers {
            keyset.extend(l.get_used_keys());
        }
        return keyset;
    }

    pub(crate) fn get_active_layers(&self) -> Vec<LayerId> {
        let mut active = Vec::new();
        for (idx, l) in (&self.layer_stack).into_iter().enumerate() {
            if l.status != LayerStatus::LayerDisabled && l.status != LayerStatus::LayerPassthrough {
                active.push(idx as LayerId);
            }
        }
        active
    }
}