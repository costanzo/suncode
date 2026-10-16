//! Running blocking work from async code without an async runtime.

use std::future::Future;

/// Runs `work` on a new thread and resolves with its result, so a caller on
/// an async executor is not stalled. `None` means the thread panicked or
/// could not be spawned.
pub fn run_blocking<T, F>(name: &str, work: F) -> impl Future<Output = Option<T>> + Send
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let (sender, receiver) = oneshot::channel();
    let spawned = std::thread::Builder::new()
        .name(name.to_string())
        .spawn(move || sender.send(work()));
    let spawned = spawned.is_ok();
    async move {
        if !spawned {
            return None;
        }
        receiver.await
    }
}

/// A minimal std-only one-shot channel. `None` means the sender was dropped,
/// for example because the operation thread panicked.
mod oneshot {
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::{Arc, Mutex};
    use std::task::{Context, Poll, Waker};

    struct State<T> {
        value: Option<T>,
        waker: Option<Waker>,
        closed: bool,
    }

    pub(super) struct Sender<T>(Arc<Mutex<State<T>>>);
    pub(super) struct Receiver<T>(Arc<Mutex<State<T>>>);

    pub(super) fn channel<T>() -> (Sender<T>, Receiver<T>) {
        let state = Arc::new(Mutex::new(State {
            value: None,
            waker: None,
            closed: false,
        }));
        (Sender(state.clone()), Receiver(state))
    }

    impl<T> Sender<T> {
        pub(super) fn send(self, value: T) {
            if let Ok(mut state) = self.0.lock() {
                state.value = Some(value);
            }
        }
    }

    impl<T> Drop for Sender<T> {
        fn drop(&mut self) {
            let waker = match self.0.lock() {
                Ok(mut state) => {
                    state.closed = true;
                    state.waker.take()
                }
                Err(_) => None,
            };
            if let Some(waker) = waker {
                waker.wake();
            }
        }
    }

    impl<T> Future for Receiver<T> {
        type Output = Option<T>;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
            let Ok(mut state) = self.0.lock() else {
                return Poll::Ready(None);
            };
            if let Some(value) = state.value.take() {
                return Poll::Ready(Some(value));
            }
            if state.closed {
                return Poll::Ready(None);
            }
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
