pub mod snapshot;
use serde::{Deserialize, Serialize};

use crate::domain::common::{
    event::Event,
    executor::{Commiter, Fetcher, SnapshotFetcher},
};

pub trait AggregateId: Clone + Serialize + for<'a> Deserialize<'a> {}

pub trait Aggregate: Sized {
    type ID: AggregateId;
    type Event: Event;
    type Command;
    fn aggregate_id(&self) -> &Self::ID;
    fn apply(state: Option<Self>, event: &Self::Event) -> Option<Self>;
    fn handle(command: &Self::Command) -> anyhow::Result<Vec<Self::Event>>;
}

struct UnCommitted<E>
where
    E: Event,
{
    version: u64,
    events: Vec<E>,
}

impl<E> UnCommitted<E>
where
    E: Event,
{
    fn empty() -> Self {
        Self {
            version: 0,
            events: vec![],
        }
    }
}
pub struct Context<A>
where
    A: Aggregate,
{
    id: A::ID,
    state: Option<A>,
    uncommitted: UnCommitted<A::Event>,
    version: u64,
}
impl<A> Context<A>
where
    A: Aggregate,
{
    fn empty(id: A::ID) -> Self {
        Self {
            id,
            state: None,
            uncommitted: UnCommitted::empty(),
            version: 0,
        }
    }
    fn as_aggregate(&self) -> Option<&A> {
        self.state.as_ref()
    }
    fn version(&self) -> u64 {
        self.version
    }
    fn handle(mut self, command: &A::Command) -> anyhow::Result<Self> {
        let events = A::handle(command)?;
        let state = events
            .iter()
            .fold(self.state, |state, event| A::apply(state, event));
        self.uncommitted.events.extend(events);
        Ok(Self {
            id: self.id,
            state,
            uncommitted: self.uncommitted,
            version: self.version,
        })
    }
    pub fn latest(mut self) -> Self {
        let start = (self.version - self.uncommitted.version) as usize;
        self.state = self.uncommitted.events[start..]
            .iter()
            .fold(self.state, |state, event| A::apply(state, event));
        self.version = self.uncommitted.version + self.uncommitted.events.len() as u64;
        self
    }
    pub fn commit(self) -> Commiter<A> {
        Commiter { ctx: self }
    }
    pub fn fetch(self, version: Option<u64>) -> Fetcher<A> {
        Fetcher {
            id: self.id,
            version,
        }
    }
    pub fn snapshot(self) -> SnapshotFetcher<A> {
        SnapshotFetcher { id: self.id }
    }
}
