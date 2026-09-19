use std::{fmt, sync::Arc};

use suncode_agent::{
    AgentEvent, AgentEventSubscription, AgentEventSubscriptionControl, EventReceiveError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubscriptionError {
    Lagged { missed: u64 },
    Closed,
    Empty,
}

impl fmt::Display for SubscriptionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lagged { missed } => {
                write!(
                    formatter,
                    "session event stream lagged by at least {missed} event(s)"
                )
            }
            Self::Closed => formatter.write_str("session event stream is closed"),
            Self::Empty => formatter.write_str("no session event is currently available"),
        }
    }
}

impl std::error::Error for SubscriptionError {}

impl From<EventReceiveError> for SubscriptionError {
    fn from(value: EventReceiveError) -> Self {
        match value {
            EventReceiveError::Lagged { missed } => Self::Lagged { missed },
            EventReceiveError::Closed => Self::Closed,
            EventReceiveError::Empty => Self::Empty,
        }
    }
}

#[derive(Clone)]
pub struct SessionEventStreamControl {
    inner: AgentEventSubscriptionControl,
}

impl SessionEventStreamControl {
    pub fn close(&self) {
        self.inner.close();
    }
}

pub struct SessionEventStream {
    session_id: String,
    inner: AgentEventSubscription,
}

impl SessionEventStream {
    pub(super) fn new(session_id: String, inner: AgentEventSubscription) -> Self {
        Self { session_id, inner }
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn control(&self) -> SessionEventStreamControl {
        SessionEventStreamControl {
            inner: self.inner.control(),
        }
    }

    pub async fn recv(&mut self) -> Result<Arc<AgentEvent>, SubscriptionError> {
        self.inner.recv().await.map_err(Into::into)
    }

    pub fn blocking_recv(&mut self) -> Result<Arc<AgentEvent>, SubscriptionError> {
        self.inner.blocking_recv().map_err(Into::into)
    }

    pub fn try_recv(&mut self) -> Result<Arc<AgentEvent>, SubscriptionError> {
        self.inner.try_recv().map_err(Into::into)
    }

    pub fn close(&self) {
        self.inner.close();
    }
}
