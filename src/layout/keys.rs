use super::types::KeymapEvent;

#[derive(Clone, Hash, Debug, PartialEq)]
pub struct KeyGroup {
    /// Sequential or a group?
    pub(super) sequential: bool,

    pub(super) keys: Vec<evdev::Key>,

    /// Key event with mask. First a key release event is sent for each mask key,
    /// then a click (press followed by release) of keys and at the end the mask
    /// is replayed as keypress events in reverse order (the same as Kg)
    pub(super) mask: Vec<evdev::Key>,
}

impl KeyGroup {
    pub fn get_used_keys(&self) -> Vec<evdev::Key> {
        let mut keys = Vec::new();
        keys.extend(&self.keys);
        keys.extend(&self.mask);
        keys
    }

    pub fn k(self, ky: evdev::Key) -> Self {
        let mut keys = Vec::from_iter(self.keys);
        keys.push(ky);

        Self {
            keys,
            ..self
        }
    }

    pub fn m(self, ky: evdev::Key) -> Self {
        let mut mask = Vec::from_iter(self.mask);
        mask.push(ky);

        Self {
            mask,
            ..self
        }
    }

    pub fn p(self) -> KeymapEvent {
        KeymapEvent::Kg(self)
    }
}

pub fn G() -> KeyGroup {
    KeyGroup {
        sequential: false,
        keys: vec![],
        mask: vec![],
    }
}

pub fn S() -> KeyGroup {
    KeyGroup {
        sequential: true,
        keys: vec![],
        mask: vec![],
    }
}