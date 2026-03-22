use anyhow::anyhow;
use ulid::Ulid;

use crate::common;

pub enum UniqueKey {
    Id(Ulid),
    Name(String),
}

impl common::aggregate::key::UniqueKey for UniqueKey {}

pub enum SearchKey {
    Id(Ulid),
    Name(String),
}

impl common::aggregate::key::SearchKey for SearchKey {}

pub struct Tag {
    pub id: Ulid,
    pub name: String,
    pub hero: Option<String>,
    pub description: String,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum Event {
    Created {
        id: Ulid,
        name: String,
        hero: Option<String>,
        description: String,
    },
    NameUpdated {
        name: String,
    },
    HeroUpdated {
        hero: Option<String>,
    },
    DescriptionUpdated {
        description: String,
    },
    Deleted,
}

impl common::event::Event for Event {}

pub enum Command {
    Create {
        id: Ulid,
        name: String,
        hero: Option<String>,
        description: String,
    },
    UpdateName {
        name: String,
    },
    UpdateHero {
        hero: Option<String>,
    },
    UpdateDescription {
        description: String,
    },
    Delete,
}

impl common::aggregate::Aggregate for Tag {
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
                hero,
                description,
            } => {
                let state = Tag {
                    id: id.clone(),
                    name: name.clone(),
                    hero: hero.clone(),
                    description: description.clone(),
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
            Event::HeroUpdated { hero } => {
                if let Some(mut state) = state {
                    state.hero = hero.clone();
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
            Event::Deleted => None,
        }
    }

    fn handle(state: Option<&Self>, command: Self::Command) -> anyhow::Result<Vec<Self::Event>> {
        match command {
            Command::Create {
                id,
                name,
                hero,
                description,
            } => {
                if state.is_some() {
                    return Err(anyhow!("aggregate already exists"));
                }
                if name.is_empty() {
                    return Err(anyhow!("name cannot be empty"));
                }
                Ok(vec![Event::Created {
                    id,
                    name,
                    hero,
                    description,
                }])
            }
            Command::UpdateName { name } => {
                let _state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                if name.is_empty() {
                    return Err(anyhow!("name cannot be empty"));
                }
                Ok(vec![Event::NameUpdated { name }])
            }
            Command::UpdateHero { hero } => {
                let _state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                Ok(vec![Event::HeroUpdated { hero }])
            }
            Command::UpdateDescription { description } => {
                let _state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                Ok(vec![Event::DescriptionUpdated { description }])
            }
            Command::Delete => {
                let _state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                Ok(vec![Event::Deleted])
            }
        }
    }
}
