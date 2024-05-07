use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub(crate) enum MessageType {
    INFO,
    WARNING,
    ERROR,
}

#[derive(Debug, Clone)]
pub(crate) struct UserMessage {
    pub(crate) msg: String,
    pub(crate) show: bool,
    pub(crate) start: Instant,
    pub(crate) duration: Duration,
    pub(crate) typ: MessageType,
}

impl UserMessage {
    pub(crate) fn welcome() -> Self {
        Self {
            msg: "Welcome to Tuipaz!".to_string(),
            show: true,
            start: Instant::now(),
            duration: Duration::from_secs(3),
            typ: MessageType::INFO,
        }
    }

    pub(crate) fn new(
        msg: String,
        show: bool,
        start: Instant,
        duration: Duration,
        typ: MessageType,
    ) -> Self {
        Self {
            msg,
            show,
            start,
            duration,
            typ,
        }
    }
}
