use anyhow::{Context, anyhow};
use domain::{
    aggregate::post::{Command, Post, UniqueKey},
    common::{
        aggregate,
        event::{EventReader, EventWriter},
        executor::Executor,
        projection::{InlineProjection, ProjectionStore},
    },
};
use ulid::Ulid;

use crate::{
    dto::post::{CreatePost, UpdatePost},
    interface::service::UserQueryService,
};

pub struct PostCase<S, U>
where
    S: EventReader + EventWriter + ProjectionStore<InlineProjection<Post>>,
    U: UserQueryService,
{
    store: S,
    user_service: U,
}

impl<S, U> PostCase<S, U>
where
    S: EventReader + EventWriter + ProjectionStore<InlineProjection<Post>>,
    U: UserQueryService,
{
    pub async fn user_create_post(&self, uid: Ulid, input: CreatePost) -> anyhow::Result<()> {
        self.user_service
            .get_summary(uid)
            .await?
            .context("author not exists")?;
        self.store
            .unique(&UniqueKey::title(uid, input.title.clone()))
            .await?
            .is_none()
            .ok_or(anyhow!("title already exists"))?;
        let post_id = Ulid::new();
        let command = Command::Create {
            id: post_id,
            title: input.title,
            content: input.content,
            author_id: uid,
        };
        aggregate::Context::<Post>::empty(post_id)
            .handle(command)?
            .commit()
            .execute(&self.store)
            .await?;
        Ok(())
    }
    pub async fn user_delete_post(&self, uid: Ulid, post_id: Ulid) -> anyhow::Result<()> {
        self.user_service
            .get_summary(uid)
            .await?
            .context("author not exists")?;
        let command = Command::AuthorOwnDelete { author_id: uid };
        aggregate::Context::<Post>::empty(post_id)
            .fetch(None)
            .execute(&self.store)
            .await?
            .handle(command)?
            .commit()
            .execute(&self.store)
            .await?;
        Ok(())
    }
    pub async fn user_update_post(
        &self,
        uid: Ulid,
        post_id: Ulid,
        input: UpdatePost,
    ) -> anyhow::Result<()> {
        self.user_service
            .get_summary(uid)
            .await?
            .context("author not exists")?;
        let command = Command::AuthorOwnUpdate {
            author_id: uid,
            title: input.title,
            content: input.content,
        };
        aggregate::Context::<Post>::empty(post_id)
            .fetch(None)
            .execute(&self.store)
            .await?
            .handle(command)?
            .commit()
            .execute(&self.store)
            .await?;
        Ok(())
    }
}
