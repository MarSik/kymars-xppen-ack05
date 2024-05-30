use enumset::{EnumSet, EnumSetType};

pub trait HasState {
    fn has_state(self) -> bool;
}

pub enum KeyStateChange<T> {
    Pressed(T),
    Released(T),
    Click(T)
}

pub struct ChangeDetector<T> where T: EnumSetType {
    state: EnumSet<T>,
    events: Vec<KeyStateChange<T>>,
}

impl <T> ChangeDetector<T> where T: EnumSetType+HasState {
    pub fn new() -> Self {
        Self {
            state: EnumSet::empty(),
            events: Vec::new(),
        }
    }

    pub fn analyze(&mut self, input: EnumSet<T>) {
        // Retrieve released keys
        for k in self.state {
            if !input.contains(k) && k.has_state() {
                self.events.push(KeyStateChange::Released(k))
            }
        }

        // Retrieve pressed keys
        for k in input {
            if !self.state.contains(k) || !k.has_state() {
                if k.has_state() {
                    self.events.push(KeyStateChange::Pressed(k))
                } else {
                    self.events.push(KeyStateChange::Click(k))
                }
            }
        }

        // Keep the last known state
        self.state = input;
    }

    pub fn next(&mut self) -> Option<KeyStateChange<T>> {
        self.events.pop()
    }
}
