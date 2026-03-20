use serde::{Deserialize, Serialize};

use crate::domain::common::aggregate::AggregateId;

pub trait Event: Clone + Serialize + for<'a> Deserialize<'a> {}

pub trait EventWriter {
    async fn commit<ID, E>(&self, stream: &ID, version: u64, events: Vec<E>) -> anyhow::Result<()>
    where
        ID: AggregateId,
        E: Event;
}

pub trait EventReader {
    async fn stream<ID, E>(
        &self,
        stream: &ID,
        version: u64,
    ) -> anyhow::Result<impl futures::Stream<Item = E> + Unpin>
    where
        ID: AggregateId,
        E: Event;
}
