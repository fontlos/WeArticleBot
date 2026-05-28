//! 飞书 WS 事件分发器

use futures_util::FutureExt;
use futures_util::future::BoxFuture;

use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

use crate::error::Result;

use super::event::EventEnvelope;

pub trait EventHandler: Send + Sync {
    fn handle(&self, envelope: EventEnvelope) -> BoxFuture<'static, Result<()>>;
}

impl<F, Fut> EventHandler for F
where
    F: Fn(EventEnvelope) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    fn handle(&self, envelope: EventEnvelope) -> BoxFuture<'static, Result<()>> {
        (self)(envelope).boxed()
    }
}

pub struct EventDispatcher {
    routes: HashMap<String, Arc<dyn EventHandler>>,
    fallback: Option<Arc<dyn EventHandler>>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            fallback: None,
        }
    }

    pub fn on<H>(&mut self, event_type: impl Into<String>, handler: H) -> &mut Self
    where
        H: EventHandler + 'static,
    {
        self.routes.insert(event_type.into(), Arc::new(handler));
        self
    }

    pub fn fallback<H>(&mut self, handler: H) -> &mut Self
    where
        H: EventHandler + 'static,
    {
        self.fallback = Some(Arc::new(handler));
        self
    }

    pub async fn dispatch_envelope(&self, envelope: EventEnvelope) -> Result<()> {
        let handler = match self.routes.get(envelope.event_type()) {
            Some(h) => h.clone(),
            None => match self.fallback.as_ref() {
                Some(h) => h.clone(),
                None => return Ok(()),
            },
        };

        handler.handle(envelope).await
    }

    pub async fn dispatch(&self, bytes: &[u8]) -> Result<()> {
        let envelope = EventEnvelope::from_bytes(bytes)?;
        self.dispatch_envelope(envelope).await
    }
}
