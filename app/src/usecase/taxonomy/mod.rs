use anyhow::Context;
use domain::{
    aggregate::taxonomy::{Command, Taxonomy, UniqueKey},
    common::{
        aggregate,
        event::{EventReader, EventWriter},
        executor::Executor,
        projection::{InlineProjection, ProjectionStore},
    },
};
use ulid::Ulid;

use crate::dto::taxonomy::{CreateTaxonomy, UpdateTaxonomy};

pub struct TaxonomyCase<S>
where
    S: EventReader + EventWriter + ProjectionStore<InlineProjection<Taxonomy>>,
{
    store: S,
}

impl<S> TaxonomyCase<S>
where
    S: EventReader + EventWriter + ProjectionStore<InlineProjection<Taxonomy>>,
{
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub async fn create(&self, input: CreateTaxonomy) -> anyhow::Result<Ulid> {
        let taxonomy_id = Ulid::new();
        let command = Command::Create {
            id: taxonomy_id,
            name: input.name,
            slug: input.slug,
            path: input.path,
            description: input.description,
            hero: input.hero,
        };
        aggregate::Context::<Taxonomy>::empty(taxonomy_id)
            .handle(command)?
            .commit()
            .execute(&self.store)
            .await?;
        Ok(taxonomy_id)
    }

    pub async fn update(
        &self,
        taxonomy_id: Ulid,
        input: UpdateTaxonomy,
    ) -> anyhow::Result<()> {
        self.require_exists(taxonomy_id).await?;
        if let Some(name) = input.name {
            let command = Command::UpdateName { name };
            aggregate::Context::<Taxonomy>::empty(taxonomy_id)
                .fetch(None)
                .execute(&self.store)
                .await?
                .handle(command)?
                .commit()
                .execute(&self.store)
                .await?;
        }
        if let Some(slug) = input.slug {
            let command = Command::UpdateSlug { slug };
            aggregate::Context::<Taxonomy>::empty(taxonomy_id)
                .fetch(None)
                .execute(&self.store)
                .await?
                .handle(command)?
                .commit()
                .execute(&self.store)
                .await?;
        }
        if let Some(path) = input.path {
            let command = Command::UpdatePath { path };
            aggregate::Context::<Taxonomy>::empty(taxonomy_id)
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
            aggregate::Context::<Taxonomy>::empty(taxonomy_id)
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
            aggregate::Context::<Taxonomy>::empty(taxonomy_id)
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

    pub async fn delete(&self, taxonomy_id: Ulid) -> anyhow::Result<()> {
        self.require_exists(taxonomy_id).await?;
        let command = Command::Delete;
        aggregate::Context::<Taxonomy>::empty(taxonomy_id)
            .fetch(None)
            .execute(&self.store)
            .await?
            .handle(command)?
            .commit()
            .execute(&self.store)
            .await?;
        Ok(())
    }

    async fn require_exists(&self, taxonomy_id: Ulid) -> anyhow::Result<()> {
        ProjectionStore::<InlineProjection<Taxonomy>>::unique(
            &self.store,
            &UniqueKey::Id(taxonomy_id),
        )
        .await?
        .context("taxonomy not exists")?;
        Ok(())
    }
}
