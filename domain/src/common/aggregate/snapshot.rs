use crate::common::{aggregate::Aggregate, executor::SnapShotLoader};

pub struct SnapShot<A>
where
    A: Aggregate,
{
    state: A,
    version: u64,
}

impl<A> SnapShot<A>
where
    A: Aggregate,
{
    fn id(&self) -> &A::ID {
        self.state.aggregate_id()
    }
    fn loader(self, version: u64) -> SnapShotLoader<A> {
        SnapShotLoader {
            snapshot: Some(self),
            version,
        }
    }
}

pub trait SnapShotStore<A>
where
    A: Aggregate,
{
    async fn get_snapshot(&self, id: &A::ID) -> anyhow::Result<SnapShot<A>>;
}
