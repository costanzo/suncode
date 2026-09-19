use std::{
    fmt,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures_core::{stream::FusedStream, Stream};

use suncode_agent::{
    AgentEvent, AgentEventSubscription, AgentEventSubscriptionControl, EventReceiveError,
};

use crate::SessionSnapshot;

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
    terminated: bool,
}

pub struct SessionWatch {
    pub snapshot: SessionSnapshot,
    pub events: SessionEventStream,
}

impl SessionEventStream {
    pub(super) fn new(session_id: String, inner: AgentEventSubscription) -> Self {
        Self {
            session_id,
            inner,
            terminated: false,
        }
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
        if self.terminated {
            return Err(SubscriptionError::Closed);
        }
        let result = self.inner.recv().await.map_err(Into::into);
        self.record_terminal_result(result)
    }

    pub fn blocking_recv(&mut self) -> Result<Arc<AgentEvent>, SubscriptionError> {
        if self.terminated {
            return Err(SubscriptionError::Closed);
        }
        let result = self.inner.blocking_recv().map_err(Into::into);
        self.record_terminal_result(result)
    }

    pub fn try_recv(&mut self) -> Result<Arc<AgentEvent>, SubscriptionError> {
        if self.terminated {
            return Err(SubscriptionError::Closed);
        }
        let result = self.inner.try_recv().map_err(Into::into);
        self.record_terminal_result(result)
    }

    pub fn close(&self) {
        self.inner.close();
    }

    fn record_terminal_result(
        &mut self,
        result: Result<Arc<AgentEvent>, SubscriptionError>,
    ) -> Result<Arc<AgentEvent>, SubscriptionError> {
        if matches!(
            result,
            Err(SubscriptionError::Lagged { .. } | SubscriptionError::Closed)
        ) {
            self.terminated = true;
            self.inner.close();
        }
        result
    }
}

impl Stream for SessionEventStream {
    type Item = Result<Arc<AgentEvent>, SubscriptionError>;

    fn poll_next(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let stream = self.get_mut();
        if stream.terminated {
            return Poll::Ready(None);
        }
        match stream
            .inner
            .poll_recv(context)
            .map(|result| result.map_err(SubscriptionError::from))
        {
            Poll::Ready(Ok(event)) => Poll::Ready(Some(Ok(event))),
            Poll::Ready(Err(SubscriptionError::Lagged { missed })) => {
                stream.terminated = true;
                stream.inner.close();
                Poll::Ready(Some(Err(SubscriptionError::Lagged { missed })))
            }
            Poll::Ready(Err(SubscriptionError::Closed)) => {
                stream.terminated = true;
                Poll::Ready(None)
            }
            Poll::Ready(Err(SubscriptionError::Empty)) => {
                Poll::Ready(Some(Err(SubscriptionError::Empty)))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl FusedStream for SessionEventStream {
    fn is_terminated(&self) -> bool {
        self.terminated
    }
}
