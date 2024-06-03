use std::collections::{HashSet, VecDeque};
use std::time::{Duration, Instant};

use evdev::Key;

use crate::kbd_events::KeyStateChange;

use super::layer::Layer;
use super::types::{KeyCoords, KeymapEvent, LayerId, LayerStatus};

const LAYER_KEY: KeyCoords = KeyCoords(255, 255, 255);

/// The key press duration threshold to distinguish between tap and hold
const HOLD_THRESHOLD_MS: Duration = Duration::from_millis(200);

pub struct LayerSwitcher {
    /// Static configuration of layers
    pub(super) layers: Vec<Layer>,
    /// Runtime status of layers
    pub(super) layer_stack: Vec<LayerStackEntry>,
    /// Currently pressed keys needing release
    /// with their originating layer and release keycodes
    pub(super) presses: Vec<(LayerId, KeyCoords, Vec<Key>, Instant)>,

    /// Queue of generated keycodes to issue to the OS
    emitted_codes: VecDeque<(Key, bool)>,
}

#[derive(Clone)]
pub struct LayerStackEntry {
    pub(super) status: LayerStatus,
    pub(super) active_keys: bool,
}

impl LayerSwitcher {
    pub fn new(layers: Vec<Layer>) -> Self {
        Self {
            layers,
            layer_stack: Vec::new(),
            presses: Vec::new(),
            emitted_codes: VecDeque::new(),
        }
    }

    /// Initialize (reset) the switcher state
    /// MUST be called before any keys are processed
    pub fn start(&mut self) {
        self.layer_stack.clear();
        for layer in &self.layers {
            self.layer_stack.push(LayerStackEntry { status: layer.status_on_reset,
                active_keys: layer.status_on_reset != LayerStatus::LayerDisabled && layer.status_on_reset != LayerStatus::LayerPassthrough })

        }
        self.layer_stack[0].status = LayerStatus::LayerActive;
        self.presses.clear();
        self.emitted_codes.clear();
    }

    /// Disable layer for good. No activation will enable it
    /// until is gets enabled explicitly.
    fn layer_disable(&mut self, idx: LayerId) {
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

    /// Set layer to passthrough and disable its rules
    fn layer_deactivate(&mut self, idx: LayerId) {
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

    /// Activate layer, keypress rules will be processed
    fn layer_activate(&mut self, idx: LayerId) {
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

    /// Activate layer and keep it activated until `coords` key is kept pressed
    fn layer_hold(&mut self, idx: LayerId, coords: KeyCoords) {
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

    /// Activate layer and keep it activated while `coords` is pressed,
    /// once `coords` is released wait for the next keypress and then deactivate
    fn layer_tap(&mut self, idx: LayerId, coords: KeyCoords) {
        // Disabled layer, ignore action
        if self.layer_stack[idx].status == LayerStatus::LayerDisabled {
            return;
        }

        // Active layer, ignore action
        if self.layer_stack[idx].status != LayerStatus::LayerPassthrough {
            return;
        }

        self.layer_stack[idx].status = LayerStatus::LayerActiveUntilKeyReleaseTap(coords);
        self.on_layer_activation(idx);
    }

    /// Activate layer `idx` and keep it activated while `coords` is pressed.
    /// At `coords` release check elapsed time and activate layer `idx2` when
    /// the press duration was shorter than `HOLD_THRESHOLD_MS`
    fn layer_hold_tap(&mut self, idx: LayerId, idx2: LayerId, coords: KeyCoords, t: Instant) {
        // Disabled layer, ignore action
        if self.layer_stack[idx].status == LayerStatus::LayerDisabled {
            return;
        }

        // Active layer, ignore action
        if self.layer_stack[idx].status != LayerStatus::LayerPassthrough {
            return;
        }

        self.layer_stack[idx].status = LayerStatus::LayerHoldAndTapToL(coords, t, idx2);
        self.on_layer_activation(idx);
    }

    /// Activate layer `idx` and keep it activated while `coords` is pressed.
    /// At `coords` release check elapsed time and emit configured keys when
    /// the press duration was shorter than `HOLD_THRESHOLD_MS`
    fn layer_hold_key(&mut self, activate_idx: LayerId, coords: KeyCoords, t: Instant, key_layer: LayerId) {
        // Disabled layer, ignore action
        if self.layer_stack[activate_idx].status == LayerStatus::LayerDisabled {
            return;
        }

        // Active layer, ignore action
        if self.layer_stack[activate_idx].status != LayerStatus::LayerPassthrough {
            return;
        }

        self.layer_stack[activate_idx].status = LayerStatus::LayerHoldAndTapKey(coords, t, key_layer);
        self.on_layer_activation(activate_idx);
    }

    /// Activate layer `idx` after all other layers were deactivated (except base layer)
    fn layer_move(&mut self, idx: LayerId) {
        // Disabled layer, ignore action
        if self.layer_stack[idx].status == LayerStatus::LayerDisabled {
            return;
        }

        for (l_idx, _l) in self.layer_stack.clone().into_iter().enumerate() {
            if idx == l_idx {
                continue;
            }
            self.layer_deactivate(idx);
        }

        self.layer_activate(idx);
    }

    /// Perform this on each layer activation
    fn on_layer_activation(&mut self, idx: LayerId) {
        let keys = self.layers[idx].on_active_keys.clone();
        for k in keys {
            self.emit_keycodes(LAYER_KEY, &k.clone(), true);
        }
        self.layer_stack[idx].active_keys = true;
    }

    /// Perform this on each layer deactivation
    fn on_layer_deactivation(&mut self, idx: LayerId) {
        // Active keys are not pressed, because some other key from the layer is active
        // and the layer is configured to disable active keys in such case
        if !self.layer_stack[idx].active_keys {
            return
        }

        let keys = self.layers[idx].on_active_keys.clone();
        for k in keys {
            self.emit_keycodes(LAYER_KEY, &k.clone(), false);
        }
    }

    /// Get the number of currently recorded presses originating from `layer`
    pub(crate) fn active_keys_from_layer(&self, layer: LayerId) -> usize {
        self.presses.iter().fold(0, |acc, (a, _, _, _)| {
            if (*a) == layer {
                acc + 1
            } else {
                acc
            }
        })
    }

    /// This is the main keypress handling function
    fn process_keyevent_press(&mut self, coords: KeyCoords, t: Instant) {
        // Identify the action associated with the current event
        let (srclayer, ev) = self.get_key_event(coords);

        // Process the event
        match ev {
            // Nothing or indirection leading nowhere
            KeymapEvent::No => {},
            KeymapEvent::Inh => {},
            KeymapEvent::Pass => {},

            KeymapEvent::K(k) => {
                if self.layers[srclayer].disable_active_on_press && self.layer_stack[srclayer].active_keys {
                    for k in (&self.layers)[srclayer].on_active_keys.clone().into_iter().rev() {
                        self.emit_keycodes(LAYER_KEY, &k, false);
                    }
                    self.layer_stack[srclayer].active_keys = false;
                }

                self.emit_keycodes(coords, &k, true);
                self.presses.push((srclayer, coords, vec![k], t));
            },
            KeymapEvent::Kg(ks) => {
                if self.layers[srclayer].disable_active_on_press && self.layer_stack[srclayer].active_keys {
                    for k in (&self.layers)[srclayer].on_active_keys.clone().into_iter().rev() {
                        self.emit_keycodes(LAYER_KEY, &k, false);
                    }
                    self.layer_stack[srclayer].active_keys = false;
                }

                // Press group
                for k in &ks {
                    self.emit_keycodes(coords, k, true);
                }
                // Keep track of pressed keys in case of layer deactivation
                self.presses.push((srclayer, coords, ks, t));
            },
            KeymapEvent::Ks(ks) => {
                for k in ks {
                    self.emit_keycodes(coords, &k, true);
                    self.emit_keycodes(coords, &k, false);
                }
            },
            KeymapEvent::Klong(..) => {
                // Record the press without a key release entry to make sure
                // the active layer is preserved
                self.presses.push((srclayer, coords, vec![], t));
            },
            KeymapEvent::Kms(km, kc) => {
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
            KeymapEvent::Lmove(idx) => self.layer_move(idx),
            KeymapEvent::Lhold(idx) => self.layer_hold(idx, coords),
            KeymapEvent::Ltap(idx) => self.layer_tap(idx, coords),
            KeymapEvent::Lactivate(idx) => self.layer_activate(idx),

            KeymapEvent::Ldisable(idx) => {
                self.layer_disable(idx);

            },
            KeymapEvent::Ldeactivate(idx) => {
                self.layer_deactivate(idx);
            },
            KeymapEvent::LhtL(idx, idx2) => self.layer_hold_tap(idx, idx2, coords, t),
            KeymapEvent::LhtK(idx, _) => self.layer_hold_key(idx, coords, t, srclayer),
            KeymapEvent::LhtKg(idx, _) => self.layer_hold_key(idx, coords, t, srclayer),
        }

        // Push forward Tap layers - a tap layer remains active only until next keypress
        for (idx, l) in self.layer_stack.clone().into_iter().enumerate() {
            if LayerStatus::LayerActiveUntilAnyKeyPress == l.status {
                self.layer_disable(idx);
            }
        }
    }

    fn process_keyevent_long_press(&mut self, coords: KeyCoords, t: Instant) {
        // Identify the action associated with the current event
        let press = self.find_press(coords);
        if press.is_none() {
            return
        }
        let press = press.unwrap();

        // Long press was still too short, wait for another one
        if t - press.3 <= HOLD_THRESHOLD_MS {
            return
        }

        // In case no release events were recorded consult the keymap and press the long keys
        if press.2.is_empty() {
            if let KeymapEvent::Klong(_, klong) = self.layers[press.1].get_key_event(coords) {
                // Remove the short press entry
                self.presses.swap_remove(press.0);

                // Emit and record the long press entry
                self.emit_keycodes(coords, &klong, true);
                self.presses.push((press.1, coords, vec![klong], t));
            }
        }
    }

    /// Find if there is an associated recorded key release entry for `coords`
    fn find_press(&self, coords: KeyCoords) -> Option<(usize, LayerId, Vec<Key>, Instant)> {
        for (idx, (layer, coord, keys, t)) in (&self.presses).into_iter().enumerate() {
            if *coord == coords {
                return Some((idx, *layer, keys.clone(), *t))
            }
        }
        return None
    }

    /// This is the main key release handling function
    fn process_keyevent_release(&mut self, coords: KeyCoords, t: Instant) {
        // Deactivate layers
        for (idx, l) in self.layer_stack.clone().into_iter().enumerate() {
            match l.status {
                LayerStatus::LayerActiveUntilKeyRelease(wait_coords) => {
                    if wait_coords == coords {
                        self.layer_deactivate(idx);
                    }
                },
                LayerStatus::LayerActiveUntilKeyReleaseTap(wait_coords) => {
                    if wait_coords == coords {
                        self.layer_stack[idx].status = LayerStatus::LayerActiveUntilAnyKeyPress;
                    }
                },
                LayerStatus::LayerHoldAndTapKey(wait_coords, t0, lidx) => {
                    if wait_coords == coords {
                        self.layer_deactivate(idx);

                        let elapsed = t - t0;
                        if elapsed < HOLD_THRESHOLD_MS {
                            let kev = self.layers[lidx].get_key_event(wait_coords);
                            match kev {
                                KeymapEvent::LhtK(_, k) => {
                                    self.emit_keycodes(LAYER_KEY, &k, true);
                                    self.emit_keycodes(LAYER_KEY, &k, false);
                                },
                                KeymapEvent::LhtKg(_, ks) => {
                                    for k in &ks {
                                        self.emit_keycodes(LAYER_KEY, &k, true);
                                    }
                                    for k in ks.into_iter().rev() {
                                        self.emit_keycodes(LAYER_KEY, &k, false);
                                    }
                                },
                                _ => {}
                            }
                        }
                    }
                },
                LayerStatus::LayerHoldAndTapToL(wait_coords, t0, next_layer) => {
                    if wait_coords == coords {
                        self.layer_deactivate(idx);

                        let elapsed = t - t0;
                        if elapsed < HOLD_THRESHOLD_MS {
                            self.layer_tap(next_layer, coords);
                            // This is the first release already, just wait for next key
                            self.layer_stack[next_layer].status = LayerStatus::LayerActiveUntilAnyKeyPress;
                        }
                    }
                },
                _ => {}
            }
        }

        // Identify the action associated with the current event
        let press = self.find_press(coords);
        if press.is_none() {
            return
        }
        let press = press.unwrap();

        // Release key if recorded as pressed (in reverse order)
        for k in (&press.2).into_iter().rev() {
            self.emit_keycodes(coords, &k, false);
        }
        self.presses.swap_remove(press.0);

        // In case no release events were recorded consult the keymap and send the short keys
        if press.2.is_empty() {
            if let KeymapEvent::Klong(kshort, _) = self.layers[press.1].get_key_event(coords) {
                self.emit_keycodes(coords, &kshort, true);
                self.emit_keycodes(coords, &kshort, false);
            }
        }

        // Reactivate on_active key when needed

        // Layer not enabled
        if self.layer_stack[press.1].status == LayerStatus::LayerDisabled || self.layer_stack[press.1].status == LayerStatus::LayerPassthrough {
            return
        }

        // Active keys are always active, no need to reactivate
        if !self.layers[press.1].disable_active_on_press {
            return
        }

        // Keys are already active, no need to reactivate
        if self.layer_stack[press.1].active_keys {
            return
        }

        // Re-enable active keys
        for k in (&self.layers)[press.1].clone().on_active_keys {
            self.emit_keycodes(LAYER_KEY, &k, true);
        }
        self.layer_stack[press.1].active_keys = true;
    }

    /// Resolve the keymap event currently mapped to key `coords`. Take into
    /// account the state of all layers and inheritance.
    /// Returns the keymap event and the layer it came from
    fn get_key_event(&self, coords: KeyCoords) -> (LayerId, KeymapEvent) {
        'layer: for (idx, l) in (&self.layer_stack).into_iter().enumerate().rev() {
            // Skip disabled layers
            if l.status == LayerStatus::LayerDisabled || l.status == LayerStatus::LayerPassthrough {
                continue;
            }

            let mut layer_idx = idx;
            loop {
                let ev = (&self.layers)[layer_idx].get_key_event(coords);
                match ev {
                    KeymapEvent::No => return (idx, ev),

                    KeymapEvent::K(_) => return (idx, ev),
                    KeymapEvent::Kg(..) => return (idx, ev),
                    KeymapEvent::Ks(_) => return (idx, ev),
                    KeymapEvent::Klong(..) => return (idx, ev),

                    KeymapEvent::Kms(..) => return (idx, ev),

                    KeymapEvent::Lmove(_) => return (idx, ev),
                    KeymapEvent::Lhold(_) => return (idx, ev),
                    KeymapEvent::Ltap(_) => return (idx, ev),
                    KeymapEvent::Lactivate(_) => return (idx, ev),
                    KeymapEvent::Ldeactivate(_) => return (idx, ev),
                    KeymapEvent::Ldisable(_) => return (idx, ev),
                    KeymapEvent::LhtL(..) => return (idx, ev),
                    KeymapEvent::LhtK(..) => return (idx, ev),
                    KeymapEvent::LhtKg(..) => return (idx, ev),

                    KeymapEvent::Inh => {
                        // find the layer this inherits from
                        if let Some(next_p_idx) = (&self.layers)[layer_idx].inherit {
                            // TODO check that the parent layer ID is valid
                            layer_idx = next_p_idx;
                        } else {
                            break // no parent
                        }
                    },
                    KeymapEvent::Pass => continue 'layer,
                }
            }
        }

        (0, KeymapEvent::No)
    }

    /// Record a keycode event to be sent to the OS
    fn emit_keycodes(&mut self, _coords: KeyCoords, k: &evdev::Key, pressed: bool) {
        self.emitted_codes.push_back((*k, pressed));
    }

    /// This is the input entrypoint for external key events. Right now everything is processed
    /// as a result of a call to this method.
    pub fn process_keyevent<T>(&mut self, ev: KeyStateChange<T>, t: impl Into<Instant>)
    where T: Into<KeyCoords>
    {
        assert!(self.layer_stack.len() > 0, "The layout engine was not started.");
        match ev {
            KeyStateChange::Pressed(k) => self.process_keyevent_press(k.into(), t.into()),
            KeyStateChange::Released(k) => self.process_keyevent_release(k.into(), t.into()),
            KeyStateChange::Click(k) => {
                let k = k.into();
                let ti = t.into();
                self.process_keyevent_press(k, ti);
                self.process_keyevent_release(k, ti);
            },
            KeyStateChange::LongPress(k) => self.process_keyevent_long_press(k.into(), t.into()),
        }
    }

    /// Consume all queued keycode events via the `renderer` closure.
    pub fn render<F>(&mut self, mut renderer: F)
    where F: FnMut(Key, bool)
    {
        while let Some(k) = self.emitted_codes.pop_front() {
            renderer(k.0, k.1)
        }
    }

    /// Parse all layers and return all keycodes that could be emitted
    /// from them. This is needed to be able to register the virtual
    /// keyboard to the OS.
    pub fn get_used_keys(&self) -> HashSet<Key> {
        let mut keyset = HashSet::new();
        for l in &self.layers {
            keyset.extend(&l.get_used_keys());
            keyset.extend(&l.on_active_keys);
        }
        return keyset;
    }

    /// Get list of currently active layers. Needed for tests.
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