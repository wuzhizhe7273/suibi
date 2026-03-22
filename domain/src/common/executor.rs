use crate::common::{
    aggregate::{
        Aggregate, Context,
        snapshot::{SnapShot, SnapShotStore},
    },
    event::{EventReader, EventWriter},
};

pub trait Executor<S> {
    type Ret;
    async fn execute(&self, store: &S) -> anyhow::Result<Self::Ret>;
}

pub struct Commiter<A>
where
    A: Aggregate,
{
    pub(crate) ctx: Context<A>,
}

impl<A, S> Executor<S> for Commiter<A>
where
    A: Aggregate,
    S: EventWriter,
{
    type Ret = Context<A>;
    async fn execute(&self, store: &S) -> anyhow::Result<Self::Ret> {
        todo!()
    }
}

pub struct Fetcher<A>
where
    A: Aggregate,
{
    pub(crate) id: A::ID,
    pub(crate) version: Option<u64>,
}

impl<A, S> Executor<S> for Fetcher<A>
where
    A: Aggregate,
    S: EventReader,
{
    type Ret = Context<A>;
    async fn execute(&self, store: &S) -> anyhow::Result<Self::Ret> {
        todo!()
    }
}

pub struct SnapshotFetcher<A>
where
    A: Aggregate,
{
    pub(crate) id: A::ID,
}

impl<A, S> Executor<S> for SnapshotFetcher<A>
where
    A: Aggregate,
    S: SnapShotStore<A>,
{
    type Ret = Option<SnapShot<A>>;
    async fn execute(&self, store: &S) -> anyhow::Result<Self::Ret> {
        todo!()
    }
}

pub struct SnapShotLoader<A>
where
    A: Aggregate,
{
    pub(crate) snapshot: Option<SnapShot<A>>,
    pub(crate) version: u64,
}

impl<A, S> Executor<S> for SnapShotLoader<A>
where
    A: Aggregate,
    S: SnapShotStore<A> + EventReader,
{
    type Ret = Context<A>;
    async fn execute(&self, store: &S) -> anyhow::Result<Self::Ret> {
        todo!()
    }
}
