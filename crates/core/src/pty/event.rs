pub enum PtyEvent {
    Data(Vec<u8>),
    Exit,
}

pub trait PtyEventHandler: Send + Sync + 'static {
    fn on_event(&self, session_id: &str, event: PtyEvent);
}
