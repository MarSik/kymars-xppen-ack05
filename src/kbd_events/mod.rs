use core::time;
use std::{collections::{HashMap, HashSet}, time::Instant};
use std::hash::Hash;
use enumset::{EnumSet, EnumSetType};

pub trait HasState {
    fn has_state(self) -> bool;
}

#[derive(Clone, Copy, Debug)]
pub enum KeyStateChange<T> {
    /// Key was pressed and is held down
    Pressed(T),
    /// Key was released
    Released(T),
    /// Key does not support state and was triggered
    Click(T),
    /// Key is still held down. This must be sent multiple
    /// times after each long press timeout elapses if
    /// the key is still in the pressed state.
    LongPress(T),
}

pub struct ChangeDetector<T> where T: EnumSetType+Hash {
    /// T -> time of press, short(F)/long(T)
    state: HashMap<T, (Instant, bool)>,
    /// Computed events that were not yet consumed
    events: Vec<KeyStateChange<T>>,
}

impl <T> ChangeDetector<T> where T: EnumSetType+Hash+HasState {
    pub fn new() -> Self {
        Self {
            state: HashMap::new(),
            events: Vec::new(),
        }
    }

    /// Time tick, checks for long presses
    pub fn tick(&mut self, t: Instant) {
        let keys = Vec::from_iter(self.state.keys().map(|k| *k));
        for k in keys {
            let (press_t, long_p) = self.state.get(&k).unwrap();
            // check press timestamp and send LongPress
            if t - *press_t > time::Duration::from_millis(200) {
                self.events.push(KeyStateChange::LongPress(k));

                if !long_p {
                    // Update the record to indicate long press was already sent
                    self.state.insert(k, (*press_t, true));
                }
            }
        }
    }

    /// Analyze keyboard state and detect Press, Release and LongPress events
    /// Return true when new key is pressed so potentially a long press
    /// timer can be set up.
    pub fn analyze(&mut self, input: EnumSet<T>, t: Instant) -> bool {
        let mut new_presses_detected = false;

        // Retrieve released keys
        for k in self.state.keys() {
            if !input.contains(*k) && k.has_state() {
                self.events.push(KeyStateChange::Released(*k))
            }
        }

        // Retrieve pressed keys
        for k in input {
            if !self.state.contains_key(&k) || !k.has_state() {
                if k.has_state() {
                    self.events.push(KeyStateChange::Pressed(k));
                    new_presses_detected = true;
                } else {
                    self.events.push(KeyStateChange::Click(k));
                }
            }

            if self.state.contains_key(&k) && k.has_state() {
                let (press_t, long_p) = self.state.get(&k).unwrap();
                // check press timestamp and send LongPress
                if t - *press_t > time::Duration::from_millis(200) {
                    self.events.push(KeyStateChange::LongPress(k));

                    if !long_p {
                        // Update the record to indicate long press was already sent
                        self.state.insert(k, (*press_t, true));
                    }
                }
            }
        }

        // Keep the last known state
        // Remove all released keys
        self.state.retain(|k, _| input.contains(*k));

        // Insert all newly pressed keys with timestamp
        for k in input {
            if !self.state.contains_key(&k) {
                self.state.insert(k, (t, false));
            }
        }

        return new_presses_detected;
    }

    pub fn next(&mut self) -> Option<KeyStateChange<T>> {
        self.events.pop()
    }

    pub fn has_pressed(&self) -> bool {
        !self.state.is_empty()
    }

    pub fn has_short_pressed(&self) -> bool {
        (&self.state).into_iter().any(|i| !i.1.1)
    }
}
