use std::u64;

use futures::StreamExt;

pub trait AggregateId: Clone {}
pub trait Event: Clone {}

pub trait Aggregate: Clone + Sized {
    type ID: AggregateId;
    type Event: Event;
    type Command;
    type Error: std::error::Error;
    fn aggregate_type() -> &'static str;
    // 应用事件，校验放在handle,apply应保证不出错
    fn apply(state: Option<Self>, event: &Self::Event) -> Option<Self>;
    fn handle_command(
        state: &Option<Self>,
        command: &Self::Command,
    ) -> Result<Vec<Self::Event>, Self::Error>;
}

pub trait EventReader {
    type Error: std::error::Error;
    // async fn last_version<A: Aggregate>(&self, id: &A::ID) -> Result<u64, Self::Error>;
    async fn stream<A: Aggregate>(
        &self,
        id: &A::ID,
        version: u64,
    ) -> Result<Box<dyn futures::Stream<Item = A::Event> + Unpin>, Self::Error>;
}

pub trait EventWriter {
    type Error: std::error::Error;
    async fn commit<A: Aggregate>(
        &self,
        id: &A::ID,
        version: u64,
        envelopes: &[A::Event],
    ) -> Result<(), Self::Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum CommitError<S>
where
    S: EventReader + EventWriter,
{
    #[error(transparent)]
    Reader(<S as EventReader>::Error),
    #[error(transparent)]
    Writer(<S as EventWriter>::Error),
    Conflict(VersionConflict),
}
impl<S> CommitError<S>
where
    S: EventReader + EventWriter,
{
    fn conflict(expected: u64, actual: u64) -> Self {
        CommitError::Conflict(VersionConflict { actual, expected })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("version conflict, expected is {expected}, actual is {actual}")]
pub struct VersionConflict {
    actual: u64,
    expected: u64,
}
impl VersionConflict {
    fn new(actual: u64, expected: u64) -> Self {
        Self { actual, expected }
    }
}
#[derive(Debug, Clone)]
pub struct UnCommitted<A>
where
    A: Aggregate,
{
    pub version: u64,
    pub events: Vec<A::Event>,
}
impl<A> Default for UnCommitted<A>
where
    A: Aggregate,
{
    fn default() -> Self {
        UnCommitted {
            version: 0,
            events: vec![],
        }
    }
}

impl<A> UnCommitted<A>
where
    A: Aggregate,
{
    fn push(&mut self, event: A::Event) {
        self.events.push(event);
    }
    fn extend(&mut self, events: Vec<A::Event>) {
        self.events.extend(events);
    }
    fn empty() -> Self {
        UnCommitted::default()
    }
}

pub struct Context<A>
where
    A: Aggregate,
{
    id: A::ID,
    aggregate: Option<A>,
    uncommitted: UnCommitted<A>,
    version: u64,
}
impl<A> Context<A>
where
    A: Aggregate,
{
    pub fn init() -> Result<(), anyhow::Result<()>> {
        todo!()
    }
    pub fn empty(id: A::ID) -> Self {
        Self {
            id,
            aggregate: None,
            uncommitted: UnCommitted::empty(),
            version: 0,
        }
    }
    pub fn handle_command(&mut self, command: &A::Command) -> Result<&mut Self, A::Error> {
        let events = A::handle_command(&self.aggregate, command)?;
        self.uncommitted.extend(events);
        Ok(self)
    }

    fn rehydrate(&self) -> Self {
        let state = self.uncommitted.events[(self.version - self.uncommitted.version) as usize..]
            .iter()
            .fold(self.aggregate.clone(), |state, event| {
                A::apply(state, &event)
            });
        Self {
            id: self.id.clone(),
            aggregate: state,
            uncommitted: self.uncommitted.clone(),
            version: self.version + self.uncommitted.events.len() as u64,
        }
    }
    async fn get<S>(mut self, store: &S) -> Result<Self, S::Error>
    where
        S: EventReader,
    {
        let mut events = store.stream::<A>(&self.id, 0).await?;
        while let Some(event) = events.next().await {
            self.aggregate = A::apply(self.aggregate, &event);
            self.version += 1;
        }
        Ok(self)
    }

    async fn commit<S>(mut self, store: &S) -> Result<Self, CommitError<S>>
    where
        S: EventReader + EventWriter,
    {
        // 提交事件
        store
            .commit::<A>(&self.id, self.uncommitted.version, &self.uncommitted.events)
            .await
            .map_err(|e| CommitError::Writer(e))?;
        // 重新计算状态
        let state = self.uncommitted.events[(self.version - self.uncommitted.version) as usize..]
            .iter()
            .fold(self.aggregate.clone(), |state, event| {
                A::apply(state, &event)
            });
        //改正状态
        self.version = self.uncommitted.version + self.uncommitted.events.len() as u64;
        self.uncommitted.version = self.version + 1;
        self.uncommitted.events.clear();
        Ok(self)
    }
}
