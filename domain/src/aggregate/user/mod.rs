use std::collections::HashSet;

use anyhow::anyhow;
use ulid::Ulid;

use crate::common;

pub enum UniqueKey {
    Id(Ulid),
    Email(String),
}

impl common::aggregate::key::UniqueKey for UniqueKey {}

pub enum SearchKey {
    Id(Ulid),
    Email(String),
}

impl common::aggregate::key::SearchKey for SearchKey {}

pub struct User {
    pub id: Ulid,
    pub username: String,
    pub email: Option<String>,
    pub phc: Option<String>,
    pub roles: HashSet<Ulid>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub enum Event {
    Created {
        id: Ulid,
        username: String,
        email: Option<String>,
        phc: Option<String>,
    },
    EmailUpdated {
        email: Option<String>,
    },
    PhcUpdated {
        phc: Option<String>,
    },
    RolesAssigned {
        roles: Vec<Ulid>,
    },
    RolesRevoked {
        roles: Vec<Ulid>,
    },
    Deleted,
}
impl common::event::Event for Event {}
pub enum Command {
    CreateByEmail {
        id: Ulid,
        username: String,
        email: String,
        phc: String,
    },
    UpdateEmail {
        email: Option<String>,
    },
    UpdatePhc {
        phc: Option<String>,
    },
    AssignRoles {
        roles: HashSet<Ulid>,
    },
    RevokeRoles {
        roles: HashSet<Ulid>,
    },
    UserDelete {
        id: Ulid,
    },
}

impl common::aggregate::Aggregate for User {
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
                username,
                email,
                phc,
            } => {
                let state = User {
                    id: id.clone(),
                    username: username.clone(),
                    email: email.clone(),
                    phc: phc.clone(),
                    roles: HashSet::new(),
                };
                Some(state)
            }
            Event::PhcUpdated { phc } => {
                if let Some(mut state) = state {
                    state.phc = phc.clone();
                    Some(state)
                } else {
                    None
                }
            }
            Event::EmailUpdated { email } => {
                if let Some(mut state) = state {
                    state.email = email.clone();
                    Some(state)
                } else {
                    None
                }
            }
            Event::Deleted => None,
            Event::RolesAssigned { roles } => {
                if let Some(mut state) = state {
                    state.roles.extend(roles.iter().cloned());
                    Some(state)
                } else {
                    None
                }
            }
            Event::RolesRevoked { roles } => {
                if let Some(mut state) = state {
                    state.roles.retain(|r| !roles.contains(r));
                    Some(state)
                } else {
                    None
                }
            }
        }
    }
    fn handle(state: Option<&Self>, command: Self::Command) -> anyhow::Result<Vec<Self::Event>> {
        match command {
            Command::CreateByEmail {
                id,
                username,
                email,
                phc,
            } => {
                if state.is_some() {
                    return Err(anyhow!("aggregate already exists"));
                }
                if username.is_empty() {
                    return Err(anyhow!("username cannot be empty"));
                }
                if email.is_empty() {
                    return Err(anyhow!("email cannot be empty"));
                }
                if phc.is_empty() {
                    return Err(anyhow!("phc cannot be empty"));
                }
                Ok(vec![Event::Created {
                    id,
                    username,
                    email: Some(email),
                    phc: Some(phc),
                }])
            }
            Command::UpdateEmail { email } => {
                let _state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                if email.as_ref().map_or(false, |s| s.is_empty()) {
                    return Err(anyhow!("email cannot be empty"));
                }
                let events = vec![Event::EmailUpdated {
                    email: email.clone(),
                }];
                Ok(events)
            }
            Command::UpdatePhc { phc } => {
                let _state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                if phc.as_ref().map_or(false, |s| s.is_empty()) {
                    return Err(anyhow!("phc cannot be empty"));
                }
                let events = vec![Event::PhcUpdated { phc: phc.clone() }];
                Ok(events)
            }
            Command::AssignRoles { roles } => {
                let state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                if roles.is_empty() {
                    return Err(anyhow!("roles cannot be empty"));
                }
                let duplicates: Vec<_> = roles
                    .iter()
                    .filter(|r| state.roles.iter().any(|existing| existing == *r))
                    .collect();
                if !duplicates.is_empty() {
                    return Err(anyhow!("roles already assigned: {:?}", duplicates));
                }
                Ok(vec![Event::RolesAssigned {
                    roles: roles.into_iter().collect(),
                }])
            }
            Command::RevokeRoles { roles } => {
                let state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                if roles.is_empty() {
                    return Err(anyhow!("roles cannot be empty"));
                }
                let not_exist: Vec<_> = roles
                    .iter()
                    .filter(|r| !state.roles.iter().any(|existing| existing == *r))
                    .collect();
                if !not_exist.is_empty() {
                    return Err(anyhow!("roles not found: {:?}", not_exist));
                }
                Ok(vec![Event::RolesRevoked {
                    roles: roles.into_iter().collect(),
                }])
            }
            Command::UserDelete { id } => {
                let state = state.ok_or_else(|| anyhow!("aggregate not found"))?;
                if state.id != id {
                    return Err(anyhow!("user not found"));
                }
                Ok(vec![Event::Deleted])
            }
        }
    }
}
