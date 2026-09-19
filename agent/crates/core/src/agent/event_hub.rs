use super::events::AgentEvent;
use std::{
    collections::HashMap,
    fmt,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex, Weak,
    },
};
use tokio::sync::mpsc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventReceiveError {
    Lagged { missed: u64 },
    Closed,
    Empty,
}

impl fmt::Display for EventReceiveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lagged { missed } => {
                write!(
                    formatter,
                    "session event subscriber lagged by at least {missed} event(s)"
                )
            }
            Self::Closed => formatter.write_str("session event subscription is closed"),
            Self::Empty => formatter.write_str("no session event is currently available"),
        }
    }
}

impl std::error::Error for EventReceiveError {}

#[derive(Default)]
struct SubscriberState {
    stale: AtomicBool,
    missed: AtomicU64,
}

struct Subscriber {
    sender: mpsc::Sender<Arc<AgentEvent>>,
    state: Arc<SubscriberState>,
}

struct SessionEventHubInner {
    capacity: usize,
    next_subscription_id: AtomicU64,
    subscribers: Mutex<HashMap<String, HashMap<u64, Subscriber>>>,
}

#[derive(Clone)]
pub struct SessionEventHub {
    inner: Arc<SessionEventHubInner>,
}

impl SessionEventHub {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "session event capacity must be positive");
        Self {
            inner: Arc::new(SessionEventHubInner {
                capacity,
                next_subscription_id: AtomicU64::new(1),
                subscribers: Mutex::new(HashMap::new()),
            }),
        }
    }

    pub fn subscribe(&self, session_id: impl Into<String>) -> AgentEventSubscription {
        let session_id = session_id.into();
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
            .entry(session_id.clone())
            .or_default()
            .insert(
                subscription_id,
                Subscriber {
                    sender,
                    state: state.clone(),
                },
            );
        let registration = Arc::new(SubscriptionRegistration {
            hub: Arc::downgrade(&self.inner),
            session_id,
            subscription_id,
            closed: AtomicBool::new(false),
            state,
        });
        AgentEventSubscription {
            receiver,
            registration,
        }
    }

    pub fn publish(&self, event: AgentEvent) {
        let event = Arc::new(event);
        let mut sessions = self
            .inner
            .subscribers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(subscribers) = sessions.get_mut(&event.session_id) else {
            return;
        };
        subscribers.retain(|_, subscriber| {
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
        if subscribers.is_empty() {
            sessions.remove(&event.session_id);
        }
    }
}

struct SubscriptionRegistration {
    hub: Weak<SessionEventHubInner>,
    session_id: String,
    subscription_id: u64,
    closed: AtomicBool,
    state: Arc<SubscriberState>,
}

impl SubscriptionRegistration {
    fn close(&self) {
        if self.closed.swap(true, Ordering::AcqRel) {
            return;
        }
        let Some(hub) = self.hub.upgrade() else {
            return;
        };
        let mut sessions = hub
            .subscribers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(subscribers) = sessions.get_mut(&self.session_id) else {
            return;
        };
        subscribers.remove(&self.subscription_id);
        if subscribers.is_empty() {
            sessions.remove(&self.session_id);
        }
    }

    fn error(&self) -> Option<EventReceiveError> {
        if self.state.stale.load(Ordering::Acquire) {
            return Some(EventReceiveError::Lagged {
                missed: self.state.missed.load(Ordering::Acquire).max(1),
            });
        }
        self.closed
            .load(Ordering::Acquire)
            .then_some(EventReceiveError::Closed)
    }
}

impl Drop for SubscriptionRegistration {
    fn drop(&mut self) {
        self.close();
    }
}

#[derive(Clone)]
pub struct AgentEventSubscriptionControl {
    registration: Arc<SubscriptionRegistration>,
}

impl AgentEventSubscriptionControl {
    pub fn close(&self) {
        self.registration.close();
    }
}

pub struct AgentEventSubscription {
    receiver: mpsc::Receiver<Arc<AgentEvent>>,
    registration: Arc<SubscriptionRegistration>,
}

impl AgentEventSubscription {
    pub fn control(&self) -> AgentEventSubscriptionControl {
        AgentEventSubscriptionControl {
            registration: self.registration.clone(),
        }
    }

    pub async fn recv(&mut self) -> Result<Arc<AgentEvent>, EventReceiveError> {
        if let Some(error) = self.registration.error() {
            return Err(error);
        }
        let event = self
            .receiver
            .recv()
            .await
            .ok_or(EventReceiveError::Closed)?;
        if let Some(error) = self.registration.error() {
            return Err(error);
        }
        Ok(event)
    }

    pub fn blocking_recv(&mut self) -> Result<Arc<AgentEvent>, EventReceiveError> {
        if let Some(error) = self.registration.error() {
            return Err(error);
        }
        let event = self
            .receiver
            .blocking_recv()
            .ok_or(EventReceiveError::Closed)?;
        if let Some(error) = self.registration.error() {
            return Err(error);
        }
        Ok(event)
    }

    pub fn try_recv(&mut self) -> Result<Arc<AgentEvent>, EventReceiveError> {
        if let Some(error) = self.registration.error() {
            return Err(error);
        }
        let event = self.receiver.try_recv().map_err(|error| match error {
            mpsc::error::TryRecvError::Empty => EventReceiveError::Empty,
            mpsc::error::TryRecvError::Disconnected => EventReceiveError::Closed,
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

impl Drop for AgentEventSubscription {
    fn drop(&mut self) {
        self.registration.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::events::{EventPayload, EventType, TurnStatePayload};

    fn turn_event(session_id: &str, turn_id: &str) -> AgentEvent {
        AgentEvent {
            session_id: session_id.into(),
            occurred_at: "2026-09-19T00:00:00.000Z".into(),
            payload: EventPayload::TurnState(TurnStatePayload {
                turn_id: turn_id.into(),
                state: "running".into(),
                model_id: Some("gpt-5.5".into()),
                submission_idempotency_key: Some("submission-1".into()),
                reason: None,
            }),
        }
    }

    #[tokio::test]
    async fn routes_only_to_the_subscribed_session() {
        let hub = SessionEventHub::new(4);
        let mut first = hub.subscribe("session-1");
        let mut second = hub.subscribe("session-2");

        hub.publish(turn_event("session-1", "turn-1"));

        let event = first.recv().await.unwrap();
        assert_eq!(event.session_id, "session-1");
        assert_eq!(event.event_type(), EventType::TurnState);
        assert!(matches!(second.try_recv(), Err(EventReceiveError::Empty)));
    }

    #[tokio::test]
    async fn marks_a_full_subscriber_as_lagged() {
        let hub = SessionEventHub::new(1);
        let mut subscription = hub.subscribe("session-1");

        hub.publish(turn_event("session-1", "turn-1"));
        hub.publish(turn_event("session-1", "turn-2"));

        assert!(matches!(
            subscription.recv().await,
            Err(EventReceiveError::Lagged { missed: 1 })
        ));
    }

    #[tokio::test]
    async fn unrelated_session_traffic_cannot_lag_a_subscriber() {
        let hub = SessionEventHub::new(1);
        let mut subscription = hub.subscribe("session-1");

        for index in 0..16 {
            hub.publish(turn_event("session-2", &format!("turn-{index}")));
        }
        hub.publish(turn_event("session-1", "turn-own"));

        let event = subscription.recv().await.unwrap();
        assert_eq!(event.session_id, "session-1");
    }

    #[test]
    fn close_control_wakes_a_blocking_receiver() {
        let hub = SessionEventHub::new(1);
        let mut subscription = hub.subscribe("session-1");
        let control = subscription.control();
        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let join = std::thread::spawn(move || {
            started_tx.send(()).unwrap();
            subscription.blocking_recv()
        });

        started_rx.recv().unwrap();
        control.close();

        assert!(matches!(
            join.join().unwrap(),
            Err(EventReceiveError::Closed)
        ));
    }
}
