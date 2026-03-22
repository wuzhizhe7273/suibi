use anyhow::Context;
use domain::{
    aggregate::tag::{Command, Tag, UniqueKey},
    common::{
        aggregate,
        event::{EventReader, EventWriter},
        executor::Executor,
        projection::{InlineProjection, ProjectionStore},
    },
};
use ulid::Ulid;

use crate::dto::tag::{CreateTag, UpdateTag};

pub struct TagCase<S>
where
    S: EventReader + EventWriter + ProjectionStore<InlineProjection<Tag>>,
{
    store: S,
}

impl<S> TagCase<S>
where
    S: EventReader + EventWriter + ProjectionStore<InlineProjection<Tag>>,
{
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub async fn create(&self, input: CreateTag) -> anyhow::Result<Ulid> {
        let tag_id = Ulid::new();
        let command = Command::Create {
            id: tag_id,
            name: input.name,
            hero: input.hero,
            description: input.description,
        };
        aggregate::Context::<Tag>::empty(tag_id)
            .handle(command)?
            .commit()
            .execute(&self.store)
            .await?;
        Ok(tag_id)
    }

    pub async fn update(&self, tag_id: Ulid, input: UpdateTag) -> anyhow::Result<()> {
        self.require_exists(tag_id).await?;
        if let Some(name) = input.name {
            let command = Command::UpdateName { name };
            aggregate::Context::<Tag>::empty(tag_id)
                .fetch(None)
                .execute(&self.store)
                .await?
                .handle(command)?
                .commit()
                .execute(&self.store)
                .await?;
        }
        if let Some(hero) = input.hero {
            let command = Command::UpdateHero { hero: Some(hero) };
            aggregate::Context::<Tag>::empty(tag_id)
                .fetch(None)
                .execute(&self.store)
                .await?
                .handle(command)?
                .commit()
                .execute(&self.store)
                .await?;
        }
        if let Some(description) = input.description {
            let command = Command::UpdateDescription { description };
            aggregate::Context::<Tag>::empty(tag_id)
                .fetch(None)
                .execute(&self.store)
                .await?
                .handle(command)?
                .commit()
                .execute(&self.store)
                .await?;
        }
        Ok(())
    }

    pub async fn delete(&self, tag_id: Ulid) -> anyhow::Result<()> {
        self.require_exists(tag_id).await?;
        let command = Command::Delete;
        aggregate::Context::<Tag>::empty(tag_id)
            .fetch(None)
            .execute(&self.store)
            .await?
            .handle(command)?
            .commit()
            .execute(&self.store)
            .await?;
        Ok(())
    }

    async fn require_exists(&self, tag_id: Ulid) -> anyhow::Result<()> {
        ProjectionStore::<InlineProjection<Tag>>::unique(&self.store, &UniqueKey::Id(tag_id))
            .await?
            .context("tag not exists")?;
        Ok(())
    }
}
