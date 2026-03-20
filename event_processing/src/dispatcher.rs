use std::sync::Arc;

use tokio::task;
use tracing::warn;

use crate::{Event, EventListener, EventSource};

// ---------------------------------------------------------------------------
// EventDispatcher
// ---------------------------------------------------------------------------

pub struct EventDispatcher {
    listeners: Vec<Arc<dyn EventListener>>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            listeners: Vec::new(),
        }
    }

    pub fn register(&mut self, listener: Arc<dyn EventListener>) {
        self.listeners.push(listener);
    }

    pub fn dispatch(&self, source: EventSource, event: Event) {
        let event = Arc::new(event);

        for listener in &self.listeners {
            let listener = Arc::clone(listener);
            let source = source.clone();
            let event = Arc::clone(&event);

            task::spawn(async move {
                if let Err(e) = listener.on_event(&source, &event).await {
                    warn!(
                        listener = listener.name(),
                        error = %e,
                        "event listener failed"
                    );
                }
            });
        }
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EventListenerError, SearchEvent};
    use async_trait::async_trait;
    use std::sync::Mutex;

    // -- RecordingListener --------------------------------------------------

    #[derive(Clone)]
    struct RecordingListener {
        events: Arc<Mutex<Vec<(String, String)>>>, // (source, event_type)
    }

    impl RecordingListener {
        fn new() -> Self {
            Self {
                events: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn recorded(&self) -> Vec<(String, String)> {
            self.events.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl EventListener for RecordingListener {
        async fn on_event(
            &self,
            source: &EventSource,
            event: &Event,
        ) -> Result<(), EventListenerError> {
            self.events
                .lock()
                .unwrap()
                .push((source.as_str().to_string(), event.event_type().to_string()));
            Ok(())
        }

        fn name(&self) -> &str {
            "recording"
        }
    }

    // -- FailingListener ----------------------------------------------------

    struct FailingListener;

    #[async_trait]
    impl EventListener for FailingListener {
        async fn on_event(
            &self,
            _source: &EventSource,
            _event: &Event,
        ) -> Result<(), EventListenerError> {
            Err(EventListenerError::Other("boom".into()))
        }

        fn name(&self) -> &str {
            "failing"
        }
    }

    // -- Helpers ------------------------------------------------------------

    fn make_search_event() -> Event {
        Event::Search(SearchEvent {
            query: "test".into(),
            search_type: "hybrid".into(),
            result_count: 3,
            league_filter: None,
            year_filter: None,
            team_filter: None,
            content_type_filter: None,
        })
    }

    // -- Tests --------------------------------------------------------------

    #[tokio::test]
    async fn single_listener_receives_event() {
        let listener = Arc::new(RecordingListener::new());
        let mut dispatcher = EventDispatcher::new();
        dispatcher.register(listener.clone());

        dispatcher.dispatch(EventSource::Web, make_search_event());

        // Give the spawned task time to complete
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let recorded = listener.recorded();
        assert_eq!(recorded.len(), 1);
        assert_eq!(recorded[0].0, "web");
        assert_eq!(recorded[0].1, "search");
    }

    #[tokio::test]
    async fn multiple_listeners_both_receive() {
        let listener1 = Arc::new(RecordingListener::new());
        let listener2 = Arc::new(RecordingListener::new());
        let mut dispatcher = EventDispatcher::new();
        dispatcher.register(listener1.clone());
        dispatcher.register(listener2.clone());

        dispatcher.dispatch(EventSource::Mcp, make_search_event());

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        assert_eq!(listener1.recorded().len(), 1);
        assert_eq!(listener2.recorded().len(), 1);
    }

    #[tokio::test]
    async fn empty_dispatcher_is_noop() {
        let dispatcher = EventDispatcher::new();
        // Should not panic
        dispatcher.dispatch(EventSource::Web, make_search_event());
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    #[tokio::test]
    async fn failing_listener_does_not_affect_others() {
        let good_listener = Arc::new(RecordingListener::new());
        let mut dispatcher = EventDispatcher::new();
        dispatcher.register(Arc::new(FailingListener));
        dispatcher.register(good_listener.clone());

        dispatcher.dispatch(EventSource::Web, make_search_event());

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        assert_eq!(good_listener.recorded().len(), 1);
    }
}
