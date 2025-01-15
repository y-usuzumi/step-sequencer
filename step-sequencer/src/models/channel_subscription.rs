use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use crossbeam::channel::{unbounded, Receiver, SendError, Sender};

use crate::id::{AutoIncrementId, AutoIncrementIdGen};

pub type ChannelEventSubscriberMap<T> = BTreeMap<AutoIncrementId, Sender<T>>;

pub struct ChannelEventSubscriptionModel<T> {
    channel_creator: Box<dyn 'static + Send + Fn() -> (Sender<T>, Receiver<T>)>,
    id_gen: RwLock<AutoIncrementIdGen>,
    subscriber_map: Arc<RwLock<ChannelEventSubscriberMap<T>>>,
}

impl<T: Clone> ChannelEventSubscriptionModel<T> {
    pub fn new(channel_creator: impl 'static + Send + Fn() -> (Sender<T>, Receiver<T>)) -> Self {
        Self {
            channel_creator: Box::new(channel_creator),
            id_gen: Default::default(),
            subscriber_map: Default::default(),
        }
    }

    pub fn subscribe(&self) -> ChannelEventSubscription<T> {
        let next_id = self.id_gen.write().unwrap().next();
        let (sender, receiver) = (self.channel_creator)();
        self.subscriber_map.write().unwrap().insert(next_id, sender);
        return ChannelEventSubscription {
            id: next_id,
            receiver,
            subscribers: self.subscriber_map.clone(),
        };
    }

    pub fn subscriber_map(&self) -> &Arc<RwLock<ChannelEventSubscriberMap<T>>> {
        &self.subscriber_map
    }

    // This is terrible
    pub fn send_all(
        subscriber_map: &Arc<RwLock<ChannelEventSubscriberMap<T>>>,
        event: T,
    ) -> Result<(), Vec<(AutoIncrementId, SendError<T>)>> {
        let mut errors = vec![];
        for (id, subscriber) in subscriber_map.read().unwrap().iter() {
            if let Err(err) = subscriber.send(event.clone()) {
                errors.push((*id, err));
            }
        }
        if errors.is_empty() {
            return Ok(());
        }
        return Err(errors);
    }
}

unsafe impl<T> Sync for ChannelEventSubscriptionModel<T> {}

impl<T> Default for ChannelEventSubscriptionModel<T> {
    fn default() -> Self {
        Self {
            channel_creator: Box::new(|| unbounded()),
            id_gen: Default::default(),
            subscriber_map: Default::default(),
        }
    }
}

pub struct ChannelEventSubscription<T> {
    pub id: AutoIncrementId,
    pub receiver: Receiver<T>,
    subscribers: Arc<RwLock<ChannelEventSubscriberMap<T>>>,
}

impl<T> Drop for ChannelEventSubscription<T> {
    fn drop(&mut self) {
        let mut subscriber_map = self.subscribers.write().unwrap();
        subscriber_map.remove(&self.id);
    }
}
