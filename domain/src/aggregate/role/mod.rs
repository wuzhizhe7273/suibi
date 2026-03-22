use std::collections::HashSet;

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

pub struct Role {
    pub id: Ulid,
    pub name: String,
    pub description: Option<String>,
    pub permissions: HashSet<String>,
    pub parent: Option<Ulid>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum Event {
    Created {
        id: Ulid,
        name: String,
        description: Option<String>,
    },
    NameUpdated {
        name: String,
    },
    DescriptionUpdated {
        description: Option<String>,
    },
    PermissionsGranted {
        permissions: Vec<String>,
    },
    PermissionsRevoked {
        permissions: Vec<String>,
    },
    ParentAssigned {
        parent: Ulid,
    },
    ParentRemoved,
    Deleted,
}
impl common::event::Event for Event {}

pub enum Command {
    Create {
        id: Ulid,
        name: String,
        description: Option<String>,
    },
    UpdateName {
        name: String,
    },
    UpdateDescription {
        description: Option<String>,
    },
    GrantPermissions {
        permissions: HashSet<String>,
    },
    RevokePermissions {
        permissions: HashSet<String>,
    },
    AssignParent {
        parent: Ulid,
    },
    RemoveParent,
}

impl common::aggregate::Aggregate for Role {
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
                description,
            } => {
                let state = Role {
                    id: id.clone(),
                    name: name.clone(),
                    description: description.clone(),
                    permissions: HashSet::new(),
                    parent: None,
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
            Event::DescriptionUpdated { description } => {
                if let Some(mut state) = state {
                    state.description = description.clone();
                    Some(state)
                } else {
                    None
                }
            }
            Event::PermissionsGranted { permissions } => {
                if let Some(mut state) = state {
                    state.permissions.extend(permissions.iter().cloned());
                    Some(state)
                } else {
                    None
                }
            }
            Event::PermissionsRevoked { permissions } => {
                if let Some(mut state) = state {
                    state.permissions.retain(|p| !permissions.contains(p));
                    Some(state)
                } else {
                    None
                }
            }
            Event::ParentAssigned { parent } => {
                if let Some(mut state) = state {
                    state.parent = Some(parent.clone());
                    Some(state)
                } else {
                    None
                }
            }
            Event::ParentRemoved => {
                if let Some(mut state) = state {
                    state.parent = None;
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
            Command::UpdateDescription { description } => {
                let _state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                Ok(vec![Event::DescriptionUpdated { description }])
            }
            Command::GrantPermissions { permissions } => {
                let state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                if permissions.is_empty() {
                    return Err(anyhow!("permissions cannot be empty"));
                }
                let duplicates: Vec<_> = permissions
                    .iter()
                    .filter(|p| state.permissions.contains(&**p))
                    .collect();
                if !duplicates.is_empty() {
                    return Err(anyhow!("permissions already granted: {:?}", duplicates));
                }
                Ok(vec![Event::PermissionsGranted {
                    permissions: permissions.into_iter().collect(),
                }])
            }
            Command::RevokePermissions { permissions } => {
                let state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                if permissions.is_empty() {
                    return Err(anyhow!("permissions cannot be empty"));
                }
                let not_exist: Vec<_> = permissions
                    .iter()
                    .filter(|p| !state.permissions.contains(&**p))
                    .collect();
                if !not_exist.is_empty() {
                    return Err(anyhow!("permissions not found: {:?}", not_exist));
                }
                Ok(vec![Event::PermissionsRevoked {
                    permissions: permissions.into_iter().collect(),
                }])
            }
            Command::AssignParent { parent } => {
                let state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                if state.parent.is_some() {
                    return Err(anyhow!("parent already assigned"));
                }
                if state.id == parent {
                    return Err(anyhow!("cannot assign self as parent"));
                }
                Ok(vec![Event::ParentAssigned { parent }])
            }
            Command::RemoveParent => {
                let state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                if state.parent.is_none() {
                    return Err(anyhow!("no parent to remove"));
                }
                Ok(vec![Event::ParentRemoved])
            }
        }
    }
}
