use std::collections::{HashSet, VecDeque};
use std::time::{Duration, Instant};

use evdev::Key;

use crate::kbd_events::KeyStateChange;

use super::keys::KeyGroup;
use super::layer::Layer;
use super::types::{KeyCoords, KeymapEvent, LayerId, LayerStatus};

const LAYER_KEY: KeyCoords = KeyCoords(255, 255, 255);

/// The key press duration threshold to distinguish between tap and hold
const HOLD_THRESHOLD_MS: Duration = Duration::from_millis(200);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyReleaseMode {
    Reverse,
    ForceClick
}

pub struct LayerSwitcher<'a> {
    /// Static configuration of layers
    pub(super) layers: &'a Vec<Layer>,
    /// Runtime status of layers
    pub(super) layer_stack: Vec<LayerStackEntry>,
    /// Currently pressed keys needing release
    /// with their originating layer and release keycodes
    pub(super) presses: Vec<(LayerId, KeyCoords, KeyReleaseMode, Option<&'a KeyGroup>, Instant)>,

    /// Queue of generated keycodes to issue to the OS
    emitted_codes: VecDeque<(Key, bool)>,
}

#[derive(Clone)]
pub struct LayerStackEntry {
    pub(super) status: LayerStatus,
    pub(super) active_keys: bool,
}

impl <'a> LayerSwitcher<'a> {
    pub fn new(layers: &'a Vec<Layer>) -> Self {
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
        for layer in self.layers {
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
        let keys = &self.layers[idx].on_active_keys;
        for k in keys {
            self.emit_keycodes(LAYER_KEY, &k, true);
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

        let keys = &self.layers[idx].on_active_keys;
        for k in keys {
            self.emit_keycodes(LAYER_KEY, k, false);
        }
    }

    fn before_key_press(&mut self, layer: LayerId) {
        if self.layers[layer].disable_active_on_press && (&self.layer_stack)[layer].active_keys {
            for k in (&self.layers[layer].on_active_keys).into_iter().rev() {
                self.emit_keycodes(LAYER_KEY, &k, false);
            }
            self.layer_stack[layer].active_keys = false;
        }
    }

    fn after_key_release(&mut self, layer: LayerId) {
        // Layer not enabled
        if self.layer_stack[layer].status == LayerStatus::LayerDisabled || self.layer_stack[layer].status == LayerStatus::LayerPassthrough {
            return
        }

        // Active keys are always active, no need to reactivate
        if !self.layers[layer].disable_active_on_press {
            return
        }

        // Keys are already active, no need to reactivate
        if self.layer_stack[layer].active_keys {
            return
        }

        // Re-enable active keys
        for k in &self.layers[layer].on_active_keys {
            self.emit_keycodes(LAYER_KEY, &k, true);
        }
        self.layer_stack[layer].active_keys = true;
    }

    fn keygroup_press(&mut self, kg: &'a KeyGroup, coords: KeyCoords, srclayer: LayerId, t: Instant, force_click: bool) {
        self.before_key_press(srclayer);
        for k in &kg.mask {
            self.emit_keycodes(coords, &k, false);
        }

        for k in &kg.keys {
            self.emit_keycodes(coords, &k, true);
            if kg.sequential {
                self.emit_keycodes(coords, &k, false);
            }
        }

        if !kg.sequential && force_click {
            for k in (&kg.keys).into_iter().rev() {
                self.emit_keycodes(coords, k, false);
            }
        }

        if kg.sequential || force_click {
            for k in (&kg.mask).into_iter().rev() {
                self.emit_keycodes(coords, k, true);
            }

            self.after_key_release(srclayer);
        } else {
            self.presses.push((srclayer, coords, KeyReleaseMode::Reverse, Some(kg), t));
        }
    }

    fn keygroup_release(&mut self, kg: &KeyGroup, coords: KeyCoords, srclayer: LayerId) {
        if kg.sequential {
            return; // sequential mode should have been released
        }

        for k in (&kg.keys).into_iter().rev() {
            self.emit_keycodes(coords, &k, false);
        }

        for k in (&kg.mask).into_iter().rev() {
            self.emit_keycodes(coords, k, true);
        }

        self.after_key_release(srclayer);
    }

    /// Get the number of currently recorded presses originating from `layer`
    pub(crate) fn active_keys_from_layer(&self, layer: LayerId) -> usize {
        self.presses.iter().fold(0, |acc, (a, _, _, _, _)| {
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
        if ev.is_none() {
            return
        }
        let ev = ev.unwrap();

        // Process the event
        match ev {
            // Nothing or indirection leading nowhere
            KeymapEvent::No => {},
            KeymapEvent::Inh => {},
            KeymapEvent::Pass => {},

            KeymapEvent::Kg(kg) => {
                self.keygroup_press(&kg, coords, srclayer, t, false);
            },
            KeymapEvent::Klong(kshort, _) => {
                // Record the press with a short key release entry
                self.presses.push((srclayer, coords, KeyReleaseMode::ForceClick, Some(kshort), t));
            },

            KeymapEvent::Khl(k, _) => {
                // Record the press with a short key release entry
                self.presses.push((srclayer, coords, KeyReleaseMode::ForceClick, Some(k), t));
            },
            KeymapEvent::Khtl(k, _) => {
                // Record the press with a short key release entry
                self.presses.push((srclayer, coords, KeyReleaseMode::ForceClick, Some(k), t));
            },

            KeymapEvent::Lmove(idx) => self.layer_move(*idx),
            KeymapEvent::Lhold(idx) => self.layer_hold(*idx, coords),
            KeymapEvent::Ltap(idx) => self.layer_tap(*idx, coords),
            KeymapEvent::Lactivate(idx) => self.layer_activate(*idx),

            KeymapEvent::Ldisable(idx) => {
                self.layer_disable(*idx);

            },
            KeymapEvent::Ldeactivate(idx) => {
                self.layer_deactivate(*idx);
            },
            KeymapEvent::LhtL(idx, idx2) => self.layer_hold_tap(*idx, *idx2, coords, t),
            KeymapEvent::LhtK(idx, _) => self.layer_hold_key(*idx, coords, t, srclayer),
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
        if t - press.4 <= HOLD_THRESHOLD_MS {
            return
        }

        // In case no release events were recorded consult the keymap and press the long keys
        match self.layers[press.1].get_key_event(coords) {
            KeymapEvent::Klong(_, klong) => {
                // When LongPress arrives for the first time, the short click is configured.
                // Replace it with the Long press.
                // When LongPress arrives for the second time, the long press is configured
                // without force_click, use that as a hint that no change is needed.
                if press.2 == KeyReleaseMode::ForceClick {
                    // Remove the short press entry
                    self.presses.swap_remove(press.0);

                    // Emit and record the long press entry
                    self.keygroup_press(&klong, coords, press.1, t, false);
                }
            },
            KeymapEvent::Khtl(_, l) => {
                // Remove the short press entry
                self.presses.swap_remove(press.0);
                self.layer_tap(*l, coords);
                self.layer_stack[*l].status = LayerStatus::LayerActiveUntilAnyKeyPress;
            },
            KeymapEvent::Khl(_, l) => {
                // Remove the short press entry
                self.presses.swap_remove(press.0);
                self.layer_activate(*l);
            },
            _ => {}
        }
    }

    /// Find if there is an associated recorded key release entry for `coords`
    fn find_press(&self, coords: KeyCoords) -> Option<(usize, LayerId, KeyReleaseMode, Option<&'a KeyGroup>, Instant)> {
        for (idx, (layer, coord, release_mode, kgroup, t)) in (&self.presses).into_iter().enumerate() {
            if *coord == coords {
                return Some((idx, *layer, *release_mode, *kgroup, *t))
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
                                    self.keygroup_press(&k, coords, lidx, t, true);
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

        // Release key if recorded as pressed
        self.presses.swap_remove(press.0);

        if let Some(kg) = press.3 {
            if press.2 == KeyReleaseMode::ForceClick {
                // consult the keymap and send the short keys as full click
                self.keygroup_press(&kg, coords, press.1, t, true);
            } else {
                self.keygroup_release(&kg, coords, press.1);
            }
        }

        // Reactivate on_active key when needed
        self.after_key_release(press.1);
    }

    fn get_key_event_inheritance(&self, coords: KeyCoords, idx: LayerId) -> (LayerId, &'a KeymapEvent) {
        let mut layer_idx = idx;
        loop {
            let ev = (&self.layers)[layer_idx].get_key_event(coords);
            match ev {
                KeymapEvent::No => return (idx, ev),

                KeymapEvent::Kg(_) => return (idx, ev),
                KeymapEvent::Klong(..) => return (idx, ev),

                KeymapEvent::Khl(..) => return (idx, ev),
                KeymapEvent::Khtl(..) => return (idx, ev),

                KeymapEvent::Lmove(_) => return (idx, ev),
                KeymapEvent::Lhold(_) => return (idx, ev),
                KeymapEvent::Ltap(_) => return (idx, ev),
                KeymapEvent::Lactivate(_) => return (idx, ev),
                KeymapEvent::Ldeactivate(_) => return (idx, ev),
                KeymapEvent::Ldisable(_) => return (idx, ev),
                KeymapEvent::LhtL(..) => return (idx, ev),
                KeymapEvent::LhtK(..) => return (idx, ev),

                KeymapEvent::Inh => {
                    // find the layer this inherits from
                    if let Some(next_p_idx) = (&self.layers)[layer_idx].inherit {
                        // TODO check that the parent layer ID is valid
                        layer_idx = next_p_idx;
                    } else {
                        break; // no parent
                    }
                },
                KeymapEvent::Pass => break,
            }
        }

        return (0, &(&self.layers)[layer_idx].default_action)
    }

    /// Resolve the keymap event currently mapped to key `coords`. Take into
    /// account the state of all layers and inheritance.
    /// Returns the keymap event and the layer it came from
    fn get_key_event(&self, coords: KeyCoords) -> (LayerId, Option<&'a KeymapEvent>) {
        'layer: for (idx, l) in (&self.layer_stack).into_iter().enumerate().rev() {
            // Skip disabled layers
            if l.status == LayerStatus::LayerDisabled || l.status == LayerStatus::LayerPassthrough {
                continue;
            }

            let (layerid, ev) = self.get_key_event_inheritance(coords, idx);
            if *ev != KeymapEvent::Pass {
                return (idx, Some(ev));
            }
        }

        (0, None)
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
        for l in self.layers {
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