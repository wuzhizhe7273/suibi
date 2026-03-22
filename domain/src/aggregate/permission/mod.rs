use anyhow::anyhow;
use ulid::Ulid;

use crate::common;

mod code;
pub use code::PermissionCode;

pub enum UniqueKey {
    Id(Ulid),
    Code(PermissionCode),
}

impl common::aggregate::key::UniqueKey for UniqueKey {}

pub enum SearchKey {
    Id(Ulid),
    Code(PermissionCode),
}

impl common::aggregate::key::SearchKey for SearchKey {}

pub struct Permission {
    pub id: Ulid,
    pub code: PermissionCode,
    pub description: Option<String>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum Event {
    Created {
        id: Ulid,
        code: PermissionCode,
        description: Option<String>,
    },
    CodeUpdated {
        code: PermissionCode,
    },
    DescriptionUpdated {
        description: Option<String>,
    },
    Deleted,
}
impl common::event::Event for Event {}

pub enum Command {
    Create {
        id: Ulid,
        code: PermissionCode,
        description: Option<String>,
    },
    UpdateCode {
        code: PermissionCode,
    },
    UpdateDescription {
        description: Option<String>,
    },
}

impl common::aggregate::Aggregate for Permission {
    type ID = Ulid;
    type Command = Command;
    type Event = Event;
    type UniqueKey = UniqueKey;
    type SearchKey = SearchKey;

    fn aggregate_id(&self) -> &Self::ID {
        &self.id
    }

    fn apply(state: Option<Self>, event: &Self::Event) -> Option<Self> {
        match event {
            Event::Created {
                id,
                code,
                description,
            } => {
                let state = Permission {
                    id: id.clone(),
                    code: code.clone(),
                    description: description.clone(),
                };
                Some(state)
            }
            Event::CodeUpdated { code } => {
                if let Some(mut state) = state {
                    state.code = code.clone();
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
                code,
                description,
            } => {
                if state.is_some() {
                    return Err(anyhow!("aggregate already exists"));
                }
                Ok(vec![Event::Created {
                    id,
                    code,
                    description,
                }])
            }
            Command::UpdateCode { code } => {
                let _state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                Ok(vec![Event::CodeUpdated { code }])
            }
            Command::UpdateDescription { description } => {
                let _state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                Ok(vec![Event::DescriptionUpdated { description }])
            }
        }
    }
}
