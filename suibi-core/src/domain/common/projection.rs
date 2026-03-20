pub trait Projection {
    type Store;
    type UniqueKeys;
    type Event;
}

pub trait ProjectionStore<P>
where
    P: Projection,
{
    fn handle(&self, event: &P::Event) -> anyhow::Result<()>;
    fn find_by_key(&self, key: &P::UniqueKeys) -> anyhow::Result<Option<P>>;
}
