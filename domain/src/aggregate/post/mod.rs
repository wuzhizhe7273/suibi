use std::collections::HashSet;

use anyhow::anyhow;
use ulid::Ulid;

use crate::common::{self, aggregate::key};

pub enum UniqueKey {
    Id(Ulid),
    Title { user: Ulid, title: String },
}
impl UniqueKey {
    pub fn title(user: Ulid, title: String) -> Self {
        Self::Title { user, title }
    }
}
impl key::UniqueKey for UniqueKey {}

pub struct Post {
    pub id: Ulid,
    pub title: String,
    pub content: String,
    pub author_id: Ulid,
    pub status: PostStatus,
    pub tag_ids: HashSet<Ulid>,
    pub taxonomy_ids: HashSet<Ulid>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum PostStatus {
    Draft,
    Published,
    Archived,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum Event {
    Created {
        id: Ulid,
        title: String,
        content: String,
        author_id: Ulid,
    },
    TitleUpdated {
        title: String,
    },
    ContentUpdated {
        content: String,
    },
    StatusChanged {
        status: PostStatus,
    },
    TagsAdded {
        tag_ids: Vec<Ulid>,
    },
    TagsRemoved {
        tag_ids: Vec<Ulid>,
    },
    TaxonomiesAdded {
        taxonomy_ids: Vec<Ulid>,
    },
    TaxonomiesRemoved {
        taxonomy_ids: Vec<Ulid>,
    },
    AuthorChanged {
        author_id: Ulid,
    },
    Deleted,
}
impl common::event::Event for Event {}

pub enum Command {
    Create {
        id: Ulid,
        title: String,
        content: String,
        author_id: Ulid,
    },
    UpdateTitle {
        title: String,
    },
    UpdateContent {
        content: String,
    },
    Publish,
    Archive,
    RevertToDraft,
    AddTags {
        tag_ids: HashSet<Ulid>,
    },
    RemoveTags {
        tag_ids: HashSet<Ulid>,
    },
    AddTaxonomies {
        taxonomy_ids: HashSet<Ulid>,
    },
    RemoveTaxonomies {
        taxonomy_ids: HashSet<Ulid>,
    },
    ChangeAuthor {
        author_id: Ulid,
    },
    AuthorOwnDelete {
        author_id: Ulid,
    },
    AuthorOwnUpdate {
        author_id: Ulid,
        title: Option<String>,
        content: Option<String>,
    },
}

impl common::aggregate::Aggregate for Post {
    type ID = Ulid;
    type Command = Command;
    type Event = Event;
    type UniqueKey = UniqueKey;
    fn aggregate_id(&self) -> &Self::ID {
        &self.id
    }

    fn apply(state: Option<Self>, event: &Self::Event) -> Option<Self> {
        match event {
            Event::Created {
                id,
                title,
                content,
                author_id,
            } => {
                let state = Post {
                    id: id.clone(),
                    title: title.clone(),
                    content: content.clone(),
                    author_id: author_id.clone(),
                    status: PostStatus::Draft,
                    tag_ids: HashSet::new(),
                    taxonomy_ids: HashSet::new(),
                };
                Some(state)
            }
            Event::TitleUpdated { title } => {
                if let Some(mut state) = state {
                    state.title = title.clone();
                    Some(state)
                } else {
                    None
                }
            }
            Event::ContentUpdated { content } => {
                if let Some(mut state) = state {
                    state.content = content.clone();
                    Some(state)
                } else {
                    None
                }
            }
            Event::StatusChanged { status } => {
                if let Some(mut state) = state {
                    state.status = status.clone();
                    Some(state)
                } else {
                    None
                }
            }
            Event::TagsAdded { tag_ids } => {
                if let Some(mut state) = state {
                    state.tag_ids.extend(tag_ids.iter().cloned());
                    Some(state)
                } else {
                    None
                }
            }
            Event::TagsRemoved { tag_ids } => {
                if let Some(mut state) = state {
                    state.tag_ids.retain(|t| !tag_ids.contains(t));
                    Some(state)
                } else {
                    None
                }
            }
            Event::TaxonomiesAdded { taxonomy_ids } => {
                if let Some(mut state) = state {
                    state.taxonomy_ids.extend(taxonomy_ids.iter().cloned());
                    Some(state)
                } else {
                    None
                }
            }
            Event::TaxonomiesRemoved { taxonomy_ids } => {
                if let Some(mut state) = state {
                    state.taxonomy_ids.retain(|t| !taxonomy_ids.contains(t));
                    Some(state)
                } else {
                    None
                }
            }
            Event::AuthorChanged { author_id } => {
                if let Some(mut state) = state {
                    state.author_id = author_id.clone();
                    Some(state)
                } else {
                    None
                }
            }
            Event::Deleted => None,
        }
    }

    fn handle(state: Option<&Self>, command: Self::Command) -> anyhow::Result<Vec<Self::Event>> {
        match command {
            Command::Create {
                id,
                title,
                content,
                author_id,
            } => {
                if state.is_some() {
                    return Err(anyhow!("aggregate already exists"));
                }
                if title.is_empty() {
                    return Err(anyhow!("title cannot be empty"));
                }
                if content.is_empty() {
                    return Err(anyhow!("content cannot be empty"));
                }
                Ok(vec![Event::Created {
                    id,
                    title,
                    content,
                    author_id,
                }])
            }
            Command::UpdateTitle { title } => {
                let _state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                if title.is_empty() {
                    return Err(anyhow!("title cannot be empty"));
                }
                Ok(vec![Event::TitleUpdated { title }])
            }
            Command::UpdateContent { content } => {
                let _state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                if content.is_empty() {
                    return Err(anyhow!("content cannot be empty"));
                }
                Ok(vec![Event::ContentUpdated { content }])
            }
            Command::Publish => {
                let state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                if matches!(state.status, PostStatus::Published) {
                    return Err(anyhow!("post already published"));
                }
                if matches!(state.status, PostStatus::Archived) {
                    return Err(anyhow!("cannot publish archived post"));
                }
                Ok(vec![Event::StatusChanged {
                    status: PostStatus::Published,
                }])
            }
            Command::Archive => {
                let state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                if matches!(state.status, PostStatus::Archived) {
                    return Err(anyhow!("post already archived"));
                }
                Ok(vec![Event::StatusChanged {
                    status: PostStatus::Archived,
                }])
            }
            Command::RevertToDraft => {
                let state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                if matches!(state.status, PostStatus::Draft) {
                    return Err(anyhow!("post already in draft"));
                }
                Ok(vec![Event::StatusChanged {
                    status: PostStatus::Draft,
                }])
            }
            Command::AddTags { tag_ids } => {
                let state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                if tag_ids.is_empty() {
                    return Err(anyhow!("tag_ids cannot be empty"));
                }
                let duplicates: Vec<_> = tag_ids
                    .iter()
                    .filter(|t| state.tag_ids.iter().any(|existing| existing == *t))
                    .collect();
                if !duplicates.is_empty() {
                    return Err(anyhow!("tag_ids already exist: {:?}", duplicates));
                }
                let new_tag_ids: Vec<_> = tag_ids.into_iter().collect();
                Ok(vec![Event::TagsAdded {
                    tag_ids: new_tag_ids,
                }])
            }
            Command::RemoveTags { tag_ids } => {
                let state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                if tag_ids.is_empty() {
                    return Err(anyhow!("tag_ids cannot be empty"));
                }
                let not_exist: Vec<_> = tag_ids
                    .iter()
                    .filter(|t| !state.tag_ids.iter().any(|existing| existing == *t))
                    .collect();
                if !not_exist.is_empty() {
                    return Err(anyhow!("tag_ids not found: {:?}", not_exist));
                }
                let tag_ids_to_remove: Vec<_> = tag_ids.into_iter().collect();
                Ok(vec![Event::TagsRemoved {
                    tag_ids: tag_ids_to_remove,
                }])
            }
            Command::AddTaxonomies { taxonomy_ids } => {
                let state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                if taxonomy_ids.is_empty() {
                    return Err(anyhow!("taxonomy_ids cannot be empty"));
                }
                let duplicates: Vec<_> = taxonomy_ids
                    .iter()
                    .filter(|t| state.taxonomy_ids.iter().any(|existing| existing == *t))
                    .collect();
                if !duplicates.is_empty() {
                    return Err(anyhow!("taxonomy_ids already exist: {:?}", duplicates));
                }
                let new_taxonomy_ids: Vec<_> = taxonomy_ids.into_iter().collect();
                Ok(vec![Event::TaxonomiesAdded {
                    taxonomy_ids: new_taxonomy_ids,
                }])
            }
            Command::RemoveTaxonomies { taxonomy_ids } => {
                let state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                if taxonomy_ids.is_empty() {
                    return Err(anyhow!("taxonomy_ids cannot be empty"));
                }
                let not_exist: Vec<_> = taxonomy_ids
                    .iter()
                    .filter(|t| !state.taxonomy_ids.iter().any(|existing| existing == *t))
                    .collect();
                if !not_exist.is_empty() {
                    return Err(anyhow!("taxonomy_ids not found: {:?}", not_exist));
                }
                let taxonomy_ids_to_remove: Vec<_> = taxonomy_ids.into_iter().collect();
                Ok(vec![Event::TaxonomiesRemoved {
                    taxonomy_ids: taxonomy_ids_to_remove,
                }])
            }
            Command::ChangeAuthor { author_id } => {
                let _state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                Ok(vec![Event::AuthorChanged { author_id }])
            }
            Command::AuthorOwnDelete { author_id } => {
                let state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                if author_id != state.author_id {
                    return Err(anyhow!("user is not the author"));
                }
                Ok(vec![Event::Deleted])
            }
            Command::AuthorOwnUpdate {
                author_id,
                title,
                content,
            } => {
                let state = state.ok_or(anyhow!("aggregate not found"))?;
                if author_id != state.author_id {
                    return Err(anyhow!("user is not the author"));
                }
                let title = title.unwrap_or(state.title.clone());
                let content = content.unwrap_or(state.content.clone());
                Ok(vec![
                    Event::TitleUpdated { title },
                    Event::ContentUpdated { content },
                ])
            }
        }
    }
}
