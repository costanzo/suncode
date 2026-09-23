use super::events::{AgentEvent, EventPayload};
use std::{
    collections::HashMap,
    fmt,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex, Weak,
    },
    task::{Context, Poll},
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
    gates: Mutex<HashMap<String, Arc<Mutex<()>>>>,
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
                gates: Mutex::new(HashMap::new()),
                subscribers: Mutex::new(HashMap::new()),
            }),
        }
    }

    pub fn subscribe(&self, session_id: impl Into<String>) -> AgentEventSubscription {
        let session_id = session_id.into();
        let gate = self.session_gate(&session_id);
        let _guard = gate.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        self.subscribe_locked(session_id)
    }

    pub fn subscribe_with_snapshot<T, E>(
        &self,
        session_id: impl Into<String>,
        snapshot: impl FnOnce() -> Result<T, E>,
    ) -> Result<(T, AgentEventSubscription), E> {
        let session_id = session_id.into();
        let gate = self.session_gate(&session_id);
        let _guard = gate.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let subscription = self.subscribe_locked(session_id);
        let snapshot = snapshot()?;
        Ok((snapshot, subscription))
    }

    pub fn publish_projected<E>(
        &self,
        session_id: &str,
        payload: EventPayload,
        project: impl FnOnce(&EventPayload) -> Result<String, E>,
    ) -> Result<String, E> {
        let gate = self.session_gate(session_id);
        let _guard = gate.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let occurred_at = project(&payload)?;
        self.publish_locked(AgentEvent {
            session_id: session_id.to_string(),
            occurred_at: occurred_at.clone(),
            payload,
        });
        Ok(occurred_at)
    }

    pub fn publish(&self, event: AgentEvent) {
        let gate = self.session_gate(&event.session_id);
        let _guard = gate.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        self.publish_locked(event);
    }

    pub fn close_all(&self) {
        self.inner
            .subscribers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
    }

    fn session_gate(&self, session_id: &str) -> Arc<Mutex<()>> {
        self.inner
            .gates
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .entry(session_id.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }

    fn subscribe_locked(&self, session_id: String) -> AgentEventSubscription {
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

    fn publish_locked(&self, event: AgentEvent) {
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

    #[cfg(test)]
    fn subscriber_count(&self, session_id: &str) -> usize {
        self.inner
            .subscribers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(session_id)
            .map(HashMap::len)
            .unwrap_or_default()
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

    pub fn poll_recv(
        &mut self,
        context: &mut Context<'_>,
    ) -> Poll<Result<Arc<AgentEvent>, EventReceiveError>> {
        if let Some(error) = self.registration.error() {
            return Poll::Ready(Err(error));
        }
        let event = match self.receiver.poll_recv(context) {
            Poll::Ready(Some(event)) => event,
            Poll::Ready(None) => return Poll::Ready(Err(EventReceiveError::Closed)),
            Poll::Pending => return Poll::Pending,
        };
        if let Some(error) = self.registration.error() {
            return Poll::Ready(Err(error));
        }
        Poll::Ready(Ok(event))
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
    use std::sync::atomic::AtomicUsize;

    fn turn_event(session_id: &str, turn_id: &str) -> AgentEvent {
        AgentEvent {
            session_id: session_id.into(),
            occurred_at: "2026-09-19T00:00:00.000Z".into(),
            payload: turn_payload(turn_id),
        }
    }

    fn turn_payload(turn_id: &str) -> EventPayload {
        EventPayload::TurnState(TurnStatePayload {
            turn_id: turn_id.into(),
            state: "running".into(),
            model_id: Some("gpt-5.5".into()),
            submission_idempotency_key: Some("submission-1".into()),
            reason: None,
        })
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

    #[tokio::test]
    async fn close_all_wakes_every_session_receiver() {
        let hub = SessionEventHub::new(1);
        let mut first = hub.subscribe("session-1");
        let mut second = hub.subscribe("session-2");

        hub.close_all();

        assert!(matches!(first.recv().await, Err(EventReceiveError::Closed)));
        assert!(matches!(
            second.recv().await,
            Err(EventReceiveError::Closed)
        ));
    }

    #[test]
    fn committed_event_before_watch_appears_only_in_snapshot() {
        let hub = SessionEventHub::new(4);
        let state = AtomicUsize::new(0);

        hub.publish_projected("session-1", turn_payload("turn-before"), |_| {
            state.store(1, Ordering::Release);
            Ok::<_, ()>("2026-09-19T00:00:00.000Z".into())
        })
        .unwrap();

        let (snapshot, mut subscription) = hub
            .subscribe_with_snapshot("session-1", || Ok::<_, ()>(state.load(Ordering::Acquire)))
            .unwrap();

        assert_eq!(snapshot, 1);
        assert!(matches!(
            subscription.try_recv(),
            Err(EventReceiveError::Empty)
        ));
    }

    #[test]
    fn event_after_watch_appears_only_in_stream() {
        let hub = SessionEventHub::new(4);
        let state = AtomicUsize::new(0);
        let (snapshot, mut subscription) = hub
            .subscribe_with_snapshot("session-1", || Ok::<_, ()>(state.load(Ordering::Acquire)))
            .unwrap();

        hub.publish_projected("session-1", turn_payload("turn-after"), |_| {
            state.store(1, Ordering::Release);
            Ok::<_, ()>("2026-09-19T00:00:00.000Z".into())
        })
        .unwrap();

        assert_eq!(snapshot, 0);
        let event = subscription.blocking_recv().unwrap();
        assert!(matches!(event.payload, EventPayload::TurnState(_)));
    }

    #[test]
    fn live_publication_waits_until_watch_snapshot_finishes() {
        let hub = SessionEventHub::new(4);
        let publisher = hub.clone();
        let (watch_entered_tx, watch_entered_rx) = std::sync::mpsc::channel();
        let (publish_attempted_tx, publish_attempted_rx) = std::sync::mpsc::channel();
        let join = std::thread::spawn(move || {
            watch_entered_rx.recv().unwrap();
            publish_attempted_tx.send(()).unwrap();
            publisher.publish(turn_event("session-1", "turn-live"));
        });

        let (_, mut subscription) = hub
            .subscribe_with_snapshot("session-1", || {
                watch_entered_tx.send(()).unwrap();
                publish_attempted_rx.recv().unwrap();
                Ok::<_, ()>(())
            })
            .unwrap();

        join.join().unwrap();
        let event = subscription.blocking_recv().unwrap();
        assert_eq!(event.session_id, "session-1");
    }

    #[test]
    fn failed_snapshot_unregisters_provisional_subscription() {
        let hub = SessionEventHub::new(4);
        let result: Result<((), AgentEventSubscription), &str> =
            hub.subscribe_with_snapshot("session-1", || Err("snapshot failed"));

        assert!(matches!(result, Err("snapshot failed")));
        assert_eq!(hub.subscriber_count("session-1"), 0);
    }

    #[test]
    fn different_session_gates_do_not_block_each_other() {
        let hub = SessionEventHub::new(4);
        let publisher = hub.clone();
        let (watch_entered_tx, watch_entered_rx) = std::sync::mpsc::channel();
        let (published_tx, published_rx) = std::sync::mpsc::channel();
        let join = std::thread::spawn(move || {
            watch_entered_rx.recv().unwrap();
            publisher.publish(turn_event("session-2", "turn-other"));
            published_tx.send(()).unwrap();
        });

        let _ = hub
            .subscribe_with_snapshot("session-1", || {
                watch_entered_tx.send(()).unwrap();
                published_rx.recv().unwrap();
                Ok::<_, ()>(())
            })
            .unwrap();

        join.join().unwrap();
    }
}
