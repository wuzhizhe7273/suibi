use crate::domain::common::aggregate::Aggregate;

pub trait Projection {
    type UniqueKey;
    type SearchKey;
    type Event;
}

pub trait ProjectionStore<P>
where
    P: Projection,
{
    fn handle(&self, event: &P::Event) -> anyhow::Result<()>;
    fn get_unique(&self, key: &P::UniqueKey) -> anyhow::Result<Option<P>>;
    fn get(&self, key: &P::SearchKey) -> anyhow::Result<Vec<P>>;
}
