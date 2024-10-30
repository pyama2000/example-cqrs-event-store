#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Event {
    id: (),
    payload: EventPayload,
}

impl Event {
    #[must_use]
    pub fn new(id: (), payload: EventPayload) -> Self {
        Self { id, payload }
    }

    /// イベントのID
    #[must_use]
    pub fn id(&self) -> &() {
        &self.id
    }

    /// イベントのペイロード
    #[must_use]
    pub fn payload(&self) -> &EventPayload {
        &self.payload
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPayload {}
