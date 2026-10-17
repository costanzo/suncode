use std::{
    collections::HashMap,
    fmt,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex, Weak,
    },
    task::{Context, Poll},
};

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttentionKind {
    PrimaryTurnCompleted,
    PrimaryTurnFailed,
    ApprovalRequested,
    QuestionAsked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentAttentionEvent {
    pub kind: AttentionKind,
    pub correlation_id: String,
    pub project_id: String,
    pub project_display_name: String,
    pub session_id: String,
    pub session_title: String,
    pub session_kind: String,
    pub parent_session_id: Option<String>,
    pub turn_id: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttentionReceiveError {
    Lagged { missed: u64 },
    Closed,
    Empty,
}

impl fmt::Display for AttentionReceiveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lagged { missed } => write!(
                formatter,
                "attention event subscriber lagged by at least {missed} event(s)"
            ),
            Self::Closed => formatter.write_str("attention event subscription is closed"),
            Self::Empty => formatter.write_str("no attention event is currently available"),
        }
    }
}

impl std::error::Error for AttentionReceiveError {}

#[derive(Default)]
struct SubscriberState {
    stale: AtomicBool,
    missed: AtomicU64,
}

struct Subscriber {
    sender: mpsc::Sender<Arc<AgentAttentionEvent>>,
    state: Arc<SubscriberState>,
}

struct AttentionEventHubInner {
    capacity: usize,
    next_subscription_id: AtomicU64,
    subscribers: Mutex<HashMap<u64, Subscriber>>,
}

#[derive(Clone)]
pub struct AttentionEventHub {
    inner: Arc<AttentionEventHubInner>,
}

impl AttentionEventHub {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "attention event capacity must be positive");
        Self {
            inner: Arc::new(AttentionEventHubInner {
                capacity,
                next_subscription_id: AtomicU64::new(1),
                subscribers: Mutex::new(HashMap::new()),
            }),
        }
    }

    pub fn subscribe(&self) -> AttentionEventSubscription {
        let subscription_id = self
            .inner
            .next_subscription_id
            .fetch_add(1, Ordering::Relaxed);
        let (sender, receiver) = mpsc::channel(self.inner.capacity);
        let state = Arc::new(SubscriberState::default());
        self.inner
            .subscribers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(
                subscription_id,
                Subscriber {
                    sender,
                    state: state.clone(),
                },
            );
        let registration = Arc::new(AttentionSubscriptionRegistration {
            hub: Arc::downgrade(&self.inner),
            subscription_id,
            closed: AtomicBool::new(false),
            state,
        });
        AttentionEventSubscription {
            receiver,
            registration,
        }
    }

    pub fn publish(&self, event: AgentAttentionEvent) {
        let event = Arc::new(event);
        self.inner
            .subscribers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .retain(|_, subscriber| {
                if subscriber.sender.is_closed() {
                    return false;
                }
                if subscriber.state.stale.load(Ordering::Acquire) {
                    subscriber.state.missed.fetch_add(1, Ordering::AcqRel);
                    return true;
                }
                match subscriber.sender.try_send(event.clone()) {
                    Ok(()) => true,
                    Err(mpsc::error::TrySendError::Full(_)) => {
                        subscriber.state.missed.fetch_add(1, Ordering::AcqRel);
                        subscriber.state.stale.store(true, Ordering::Release);
                        true
                    }
                    Err(mpsc::error::TrySendError::Closed(_)) => false,
                }
            });
    }

    pub fn close_all(&self) {
        self.inner
            .subscribers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
    }
}

struct AttentionSubscriptionRegistration {
    hub: Weak<AttentionEventHubInner>,
    subscription_id: u64,
    closed: AtomicBool,
    state: Arc<SubscriberState>,
}

impl AttentionSubscriptionRegistration {
    fn close(&self) {
        if self.closed.swap(true, Ordering::AcqRel) {
            return;
        }
        if let Some(hub) = self.hub.upgrade() {
            hub.subscribers
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .remove(&self.subscription_id);
        }
    }

    fn error(&self) -> Option<AttentionReceiveError> {
        if self.state.stale.load(Ordering::Acquire) {
            return Some(AttentionReceiveError::Lagged {
                missed: self.state.missed.load(Ordering::Acquire).max(1),
            });
        }
        self.closed
            .load(Ordering::Acquire)
            .then_some(AttentionReceiveError::Closed)
    }
}

impl Drop for AttentionSubscriptionRegistration {
    fn drop(&mut self) {
        self.close();
    }
}

#[derive(Clone)]
pub struct AttentionEventSubscriptionControl {
    registration: Arc<AttentionSubscriptionRegistration>,
}

impl AttentionEventSubscriptionControl {
    pub fn close(&self) {
        self.registration.close();
    }
}

pub struct AttentionEventSubscription {
    receiver: mpsc::Receiver<Arc<AgentAttentionEvent>>,
    registration: Arc<AttentionSubscriptionRegistration>,
}

impl AttentionEventSubscription {
    pub fn control(&self) -> AttentionEventSubscriptionControl {
        AttentionEventSubscriptionControl {
            registration: self.registration.clone(),
        }
    }

    pub async fn recv(&mut self) -> Result<Arc<AgentAttentionEvent>, AttentionReceiveError> {
        if let Some(error) = self.registration.error() {
            return Err(error);
        }
        let event = self
            .receiver
            .recv()
            .await
            .ok_or(AttentionReceiveError::Closed)?;
        if let Some(error) = self.registration.error() {
            return Err(error);
        }
        Ok(event)
    }

    pub fn poll_recv(
        &mut self,
        context: &mut Context<'_>,
    ) -> Poll<Result<Arc<AgentAttentionEvent>, AttentionReceiveError>> {
        if let Some(error) = self.registration.error() {
            return Poll::Ready(Err(error));
        }
        let event = match self.receiver.poll_recv(context) {
            Poll::Ready(Some(event)) => event,
            Poll::Ready(None) => return Poll::Ready(Err(AttentionReceiveError::Closed)),
            Poll::Pending => return Poll::Pending,
        };
        if let Some(error) = self.registration.error() {
            return Poll::Ready(Err(error));
        }
        Poll::Ready(Ok(event))
    }

    pub fn blocking_recv(&mut self) -> Result<Arc<AgentAttentionEvent>, AttentionReceiveError> {
        if let Some(error) = self.registration.error() {
            return Err(error);
        }
        let event = self
            .receiver
            .blocking_recv()
            .ok_or(AttentionReceiveError::Closed)?;
        if let Some(error) = self.registration.error() {
            return Err(error);
        }
        Ok(event)
    }

    pub fn try_recv(&mut self) -> Result<Arc<AgentAttentionEvent>, AttentionReceiveError> {
        if let Some(error) = self.registration.error() {
            return Err(error);
        }
        let event = self.receiver.try_recv().map_err(|error| match error {
            mpsc::error::TryRecvError::Empty => AttentionReceiveError::Empty,
            mpsc::error::TryRecvError::Disconnected => AttentionReceiveError::Closed,
        })?;
        if let Some(error) = self.registration.error() {
            return Err(error);
        }
        Ok(event)
    }

    pub fn close(&self) {
        self.registration.close();
    }
}

impl Drop for AttentionEventSubscription {
    fn drop(&mut self) {
        self.registration.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(correlation_id: &str) -> AgentAttentionEvent {
        AgentAttentionEvent {
            kind: AttentionKind::PrimaryTurnCompleted,
            correlation_id: correlation_id.into(),
            project_id: "project-1".into(),
            project_display_name: "Project".into(),
            session_id: "session-1".into(),
            session_title: "Session".into(),
            session_kind: "primary".into(),
            parent_session_id: None,
            turn_id: correlation_id.into(),
            occurred_at: "2026-09-23T00:00:00.000Z".into(),
        }
    }

    #[tokio::test]
    async fn publishes_to_every_attention_subscriber() {
        let hub = AttentionEventHub::new(4);
        let mut first = hub.subscribe();
        let mut second = hub.subscribe();
        hub.publish(event("turn-1"));
        assert_eq!(first.recv().await.unwrap().correlation_id, "turn-1");
        assert_eq!(second.recv().await.unwrap().correlation_id, "turn-1");
    }

    #[tokio::test]
    async fn reports_lag_and_closes_all() {
        let hub = AttentionEventHub::new(1);
        let mut lagged = hub.subscribe();
        hub.publish(event("turn-1"));
        hub.publish(event("turn-2"));
        assert!(matches!(
            lagged.recv().await,
            Err(AttentionReceiveError::Lagged { missed: 1 })
        ));

        let mut closed = hub.subscribe();
        hub.close_all();
        assert!(matches!(
            closed.recv().await,
            Err(AttentionReceiveError::Closed)
        ));
    }
}
