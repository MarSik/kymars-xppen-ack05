use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub(super) struct TestTime {
    t: Instant
}

impl TestTime {
    pub(super) fn start() -> Self {
        Self {
            t: Instant::now(),
        }
    }

    pub(super) fn now(&self) -> Instant {
        self.t
    }

    pub(super) fn advance_ms(&mut self, ms: u64) -> Instant {
        self.t = self.t + Duration::from_millis(ms);
        self.t
    }
}

impl Into<Instant> for TestTime {
    fn into(self) -> Instant {
        self.t
    }
}