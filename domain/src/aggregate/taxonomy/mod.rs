mod path;

use anyhow::anyhow;
use ulid::Ulid;

use crate::common;

pub use path::TaxonomyPath;

pub enum UniqueKey {
    Id(Ulid),
    Slug(String),
}

impl common::aggregate::key::UniqueKey for UniqueKey {}

pub enum SearchKey {
    Id(Ulid),
    Slug(String),
    Path(TaxonomyPath),
}

impl common::aggregate::key::SearchKey for SearchKey {}

pub struct Taxonomy {
    pub id: Ulid,
    pub name: String,
    pub slug: String,
    pub path: TaxonomyPath,
    pub description: String,
    pub hero: Option<String>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum Event {
    Created {
        id: Ulid,
        name: String,
        slug: String,
        path: TaxonomyPath,
        description: String,
        hero: Option<String>,
    },
    NameUpdated {
        name: String,
    },
    SlugUpdated {
        slug: String,
    },
    PathUpdated {
        path: TaxonomyPath,
    },
    DescriptionUpdated {
        description: String,
    },
    HeroUpdated {
        hero: Option<String>,
    },
    Deleted,
}

impl common::event::Event for Event {}

pub enum Command {
    Create {
        id: Ulid,
        name: String,
        slug: String,
        path: TaxonomyPath,
        description: String,
        hero: Option<String>,
    },
    UpdateName {
        name: String,
    },
    UpdateSlug {
        slug: String,
    },
    UpdatePath {
        path: TaxonomyPath,
    },
    UpdateDescription {
        description: String,
    },
    UpdateHero {
        hero: Option<String>,
    },
    Delete,
}

impl common::aggregate::Aggregate for Taxonomy {
    type ID = Ulid;
    type Command = Command;
    type Event = Event;
    type UniqueKey = UniqueKey;
    type SearchKey = SearchKey;

    fn aggregate_id(&self) -> &Self::ID {
        &self.id
    }

    fn apply(mut state: Option<Self>, event: &Self::Event) -> Option<Self> {
        match event {
            Event::Created {
                id,
                name,
                slug,
                path,
                description,
                hero,
            } => {
                let state = Taxonomy {
                    id: id.clone(),
                    name: name.clone(),
                    slug: slug.clone(),
                    path: path.clone(),
                    description: description.clone(),
                    hero: hero.clone(),
                };
                Some(state)
            }
            Event::NameUpdated { name } => {
                if let Some(mut state) = state {
                    state.name = name.clone();
                    Some(state)
                } else {
                    None
                }
            }
            Event::SlugUpdated { slug } => {
                if let Some(mut state) = state {
                    state.slug = slug.clone();
                    Some(state)
                } else {
                    None
                }
            }
            Event::PathUpdated { path } => {
                if let Some(mut state) = state {
                    state.path = path.clone();
                    Some(state)
                } else {
                    None
                }
            }
            Event::DescriptionUpdated { description } => {
                if let Some(mut state) = state {
                    state.description = description.clone();
                    Some(state)
                } else {
                    None
                }
            }
            Event::HeroUpdated { hero } => {
                if let Some(mut state) = state {
                    state.hero = hero.clone();
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
                name,
                slug,
                path,
                description,
                hero,
            } => {
                if state.is_some() {
                    return Err(anyhow!("aggregate already exists"));
                }
                if name.is_empty() {
                    return Err(anyhow!("name cannot be empty"));
                }
                if slug.is_empty() {
                    return Err(anyhow!("slug cannot be empty"));
                }
                Ok(vec![Event::Created {
                    id,
                    name,
                    slug,
                    path,
                    description,
                    hero,
                }])
            }
            Command::UpdateName { name } => {
                let _state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                if name.is_empty() {
                    return Err(anyhow!("name cannot be empty"));
                }
                Ok(vec![Event::NameUpdated { name }])
            }
            Command::UpdateSlug { slug } => {
                let _state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                if slug.is_empty() {
                    return Err(anyhow!("slug cannot be empty"));
                }
                Ok(vec![Event::SlugUpdated { slug }])
            }
            Command::UpdatePath { path } => {
                let _state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                Ok(vec![Event::PathUpdated { path }])
            }
            Command::UpdateDescription { description } => {
                let _state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                Ok(vec![Event::DescriptionUpdated { description }])
            }
            Command::UpdateHero { hero } => {
                let _state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                Ok(vec![Event::HeroUpdated { hero }])
            }
            Command::Delete => {
                let _state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                Ok(vec![Event::Deleted])
            }
        }
    }
}
