use crate::common::aggregate::Aggregate;

pub trait Projection {
    type UniqueKey;
    type SearchKey;
    type Event;
}

pub trait ProjectionStore<P>
where
    P: Projection,
{
    async fn handle(&self, event: &P::Event) -> anyhow::Result<()>;
    async fn unique(&self, key: &P::UniqueKey) -> anyhow::Result<Option<P>>;
    async fn get(&self, key: &P::SearchKey) -> anyhow::Result<Vec<P>>;
}

pub struct InlineProjection<A>
where
    A: Aggregate,
{
    pub(crate) state: A,
}

impl<A> Projection for InlineProjection<A>
where
    A: Aggregate,
{
    type UniqueKey = A::UniqueKey;
    type SearchKey = A::SearchKey;
    type Event = A::Event;
}

impl<A> std::ops::Deref for InlineProjection<A>
where
    A: Aggregate,
{
    type Target = A;
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}
