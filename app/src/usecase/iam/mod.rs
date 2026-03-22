use anyhow::{Context, anyhow};
use domain::{
    aggregate::{
        permission::PermissionCode,
        role::{Role, UniqueKey as RoleUniqueKey},
        user::{SearchKey as UserSearchKey, UniqueKey as UserUniqueKey, User},
    },
    common::{
        aggregate,
        event::{EventReader, EventWriter},
        executor::Executor,
        projection::{InlineProjection, ProjectionStore},
    },
};
use ulid::Ulid;

use crate::dto::iam::{
    CreatePermission, CreateRole, CreateUser, GrantPermissionsInput, GrantRolesInput, LoginInput,
    RegisterInput, RevokePermissionsInput, RevokeRolesInput, UpdatePermission, UpdateRole,
    UpdateUser,
};
use crate::interface::service::AuthService;

pub struct IAMCase<S, A>
where
    S: EventReader
        + EventWriter
        + ProjectionStore<InlineProjection<User>>
        + ProjectionStore<InlineProjection<Role>>
        + ProjectionStore<InlineProjection<domain::aggregate::permission::Permission>>,
    A: AuthService,
{
    store: S,
    auth_service: A,
}

impl<S, A> IAMCase<S, A>
where
    S: EventReader
        + EventWriter
        + ProjectionStore<InlineProjection<User>>
        + ProjectionStore<InlineProjection<Role>>
        + ProjectionStore<InlineProjection<domain::aggregate::permission::Permission>>,
    A: AuthService,
{
    pub fn new(store: S, auth_service: A) -> Self {
        Self {
            store,
            auth_service,
        }
    }

    pub async fn register(&self, input: RegisterInput) -> anyhow::Result<Ulid> {
        let phc = self.auth_service.hash_password(&input.password)?;
        let user_id = Ulid::new();
        let command = domain::aggregate::user::Command::CreateByEmail {
            id: user_id,
            username: input.username,
            email: input.email,
            phc,
        };
        aggregate::Context::<User>::empty(user_id)
            .handle(command)?
            .commit()
            .execute(&self.store)
            .await?;
        Ok(user_id)
    }

    pub async fn login(&self, input: LoginInput) -> anyhow::Result<Option<Ulid>> {
        let users = ProjectionStore::<InlineProjection<User>>::get(
            &self.store,
            &UserSearchKey::Email(input.email),
        )
        .await?;
        let user_proj = match users.into_iter().next() {
            Some(u) => u,
            None => return Ok(None),
        };
        let user: &User = &user_proj;
        let phc = match &user.phc {
            Some(p) => p,
            None => return Ok(None),
        };
        if self.auth_service.verify_password(&input.password, phc)? {
            Ok(Some(user.id))
        } else {
            Ok(None)
        }
    }

    pub async fn user_create(&self, input: CreateUser) -> anyhow::Result<Ulid> {
        let user_id = Ulid::new();
        let command = domain::aggregate::user::Command::CreateByEmail {
            id: user_id,
            username: input.username,
            email: input.email,
            phc: input.phc,
        };
        aggregate::Context::<User>::empty(user_id)
            .handle(command)?
            .commit()
            .execute(&self.store)
            .await?;
        Ok(user_id)
    }

    pub async fn user_update(&self, user_id: Ulid, input: UpdateUser) -> anyhow::Result<()> {
        self.require_user_exists(user_id).await?;
        if let Some(email) = input.email {
            let command = domain::aggregate::user::Command::UpdateEmail { email: Some(email) };
            aggregate::Context::<User>::empty(user_id)
                .fetch(None)
                .execute(&self.store)
                .await?
                .handle(command)?
                .commit()
                .execute(&self.store)
                .await?;
        }
        if let Some(phc) = input.phc {
            let command = domain::aggregate::user::Command::UpdatePhc { phc: Some(phc) };
            aggregate::Context::<User>::empty(user_id)
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

    pub async fn user_delete(&self, user_id: Ulid) -> anyhow::Result<()> {
        self.require_user_exists(user_id).await?;
        let command = domain::aggregate::user::Command::UserDelete { id: user_id };
        aggregate::Context::<User>::empty(user_id)
            .fetch(None)
            .execute(&self.store)
            .await?
            .handle(command)?
            .commit()
            .execute(&self.store)
            .await?;
        Ok(())
    }

    pub async fn user_grant_roles(
        &self,
        user_id: Ulid,
        input: GrantRolesInput,
    ) -> anyhow::Result<()> {
        self.require_user_exists(user_id).await?;
        let role_ids: std::collections::HashSet<Ulid> = input
            .roles
            .iter()
            .filter_map(|r| Ulid::from_string(r).ok())
            .collect();
        let command = domain::aggregate::user::Command::AssignRoles { roles: role_ids };
        aggregate::Context::<User>::empty(user_id)
            .fetch(None)
            .execute(&self.store)
            .await?
            .handle(command)?
            .commit()
            .execute(&self.store)
            .await?;
        Ok(())
    }

    pub async fn user_revoke_roles(
        &self,
        user_id: Ulid,
        input: RevokeRolesInput,
    ) -> anyhow::Result<()> {
        self.require_user_exists(user_id).await?;
        let role_ids: std::collections::HashSet<Ulid> = input
            .roles
            .iter()
            .filter_map(|r| Ulid::from_string(r).ok())
            .collect();
        let command = domain::aggregate::user::Command::RevokeRoles { roles: role_ids };
        aggregate::Context::<User>::empty(user_id)
            .fetch(None)
            .execute(&self.store)
            .await?
            .handle(command)?
            .commit()
            .execute(&self.store)
            .await?;
        Ok(())
    }

    pub async fn role_create(&self, input: CreateRole) -> anyhow::Result<Ulid> {
        let role_id = Ulid::new();
        let command = domain::aggregate::role::Command::Create {
            id: role_id,
            name: input.name,
            description: input.description,
        };
        aggregate::Context::<Role>::empty(role_id)
            .handle(command)?
            .commit()
            .execute(&self.store)
            .await?;
        Ok(role_id)
    }

    pub async fn role_update(&self, role_id: Ulid, input: UpdateRole) -> anyhow::Result<()> {
        self.require_role_exists(role_id).await?;
        if let Some(name) = input.name {
            let command = domain::aggregate::role::Command::UpdateName { name };
            aggregate::Context::<Role>::empty(role_id)
                .fetch(None)
                .execute(&self.store)
                .await?
                .handle(command)?
                .commit()
                .execute(&self.store)
                .await?;
        }
        if let Some(description) = input.description {
            let command = domain::aggregate::role::Command::UpdateDescription {
                description: Some(description),
            };
            aggregate::Context::<Role>::empty(role_id)
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

    pub async fn role_delete(&self, role_id: Ulid) -> anyhow::Result<()> {
        self.require_role_exists(role_id).await?;
        let _ = aggregate::Context::<Role>::empty(role_id)
            .fetch(None)
            .execute(&self.store)
            .await?
            .handle(domain::aggregate::role::Command::RemoveParent)?
            .commit()
            .execute(&self.store)
            .await;
        let _ = aggregate::Context::<Role>::empty(role_id)
            .fetch(None)
            .execute(&self.store)
            .await?
            .handle(domain::aggregate::role::Command::RevokePermissions {
                permissions: std::collections::HashSet::new(),
            })?
            .commit()
            .execute(&self.store)
            .await;
        Ok(())
    }

    pub async fn role_grant_permissions(
        &self,
        role_id: Ulid,
        input: GrantPermissionsInput,
    ) -> anyhow::Result<()> {
        self.require_role_exists(role_id).await?;
        let permissions: std::collections::HashSet<String> =
            input.permissions.into_iter().collect();
        let command = domain::aggregate::role::Command::GrantPermissions { permissions };
        aggregate::Context::<Role>::empty(role_id)
            .fetch(None)
            .execute(&self.store)
            .await?
            .handle(command)?
            .commit()
            .execute(&self.store)
            .await?;
        Ok(())
    }

    pub async fn role_revoke_permissions(
        &self,
        role_id: Ulid,
        input: RevokePermissionsInput,
    ) -> anyhow::Result<()> {
        self.require_role_exists(role_id).await?;
        let permissions: std::collections::HashSet<String> =
            input.permissions.into_iter().collect();
        let command = domain::aggregate::role::Command::RevokePermissions { permissions };
        aggregate::Context::<Role>::empty(role_id)
            .fetch(None)
            .execute(&self.store)
            .await?
            .handle(command)?
            .commit()
            .execute(&self.store)
            .await?;
        Ok(())
    }

    pub async fn permission_create(&self, input: CreatePermission) -> anyhow::Result<Ulid> {
        let permission_id = Ulid::new();
        let code = PermissionCode::new(input.code)?;
        let command = domain::aggregate::permission::Command::Create {
            id: permission_id,
            code,
            description: input.description,
        };
        aggregate::Context::<domain::aggregate::permission::Permission>::empty(permission_id)
            .handle(command)?
            .commit()
            .execute(&self.store)
            .await?;
        Ok(permission_id)
    }

    pub async fn permission_update(
        &self,
        permission_id: Ulid,
        input: UpdatePermission,
    ) -> anyhow::Result<()> {
        self.require_permission_exists(permission_id).await?;
        if let Some(code) = input.code {
            let code = PermissionCode::new(code)?;
            let command = domain::aggregate::permission::Command::UpdateCode { code };
            aggregate::Context::<domain::aggregate::permission::Permission>::empty(permission_id)
                .fetch(None)
                .execute(&self.store)
                .await?
                .handle(command)?
                .commit()
                .execute(&self.store)
                .await?;
        }
        if let Some(description) = input.description {
            let command = domain::aggregate::permission::Command::UpdateDescription {
                description: Some(description),
            };
            aggregate::Context::<domain::aggregate::permission::Permission>::empty(permission_id)
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

    pub async fn permission_delete(&self, permission_id: Ulid) -> anyhow::Result<()> {
        self.require_permission_exists(permission_id).await?;
        let command =
            domain::aggregate::permission::Command::UpdateDescription { description: None };
        aggregate::Context::<domain::aggregate::permission::Permission>::empty(permission_id)
            .fetch(None)
            .execute(&self.store)
            .await?
            .handle(command)?
            .commit()
            .execute(&self.store)
            .await?;
        Ok(())
    }

    async fn require_user_exists(&self, user_id: Ulid) -> anyhow::Result<()> {
        ProjectionStore::<InlineProjection<User>>::unique(&self.store, &UserUniqueKey::Id(user_id))
            .await?
            .context("user not exists")?;
        Ok(())
    }

    async fn require_role_exists(&self, role_id: Ulid) -> anyhow::Result<()> {
        ProjectionStore::<InlineProjection<Role>>::unique(&self.store, &RoleUniqueKey::Id(role_id))
            .await?
            .context("role not exists")?;
        Ok(())
    }

    async fn require_permission_exists(&self, permission_id: Ulid) -> anyhow::Result<()> {
        ProjectionStore::<InlineProjection<domain::aggregate::permission::Permission>>::unique(
            &self.store,
            &domain::aggregate::permission::UniqueKey::Id(permission_id),
        )
        .await?
        .context("permission not exists")?;
        Ok(())
    }

    pub async fn user_has_role(&self, user_id: Ulid, role_id: Ulid) -> anyhow::Result<bool> {
        let user_proj = match ProjectionStore::<InlineProjection<User>>::unique(
            &self.store,
            &UserUniqueKey::Id(user_id),
        )
        .await?
        {
            Some(u) => u,
            None => return Ok(false),
        };
        let user: &User = &user_proj;
        Ok(user.roles.contains(&role_id))
    }

    pub async fn user_has_permission(
        &self,
        user_id: Ulid,
        permission_code: &str,
    ) -> anyhow::Result<bool> {
        let user_proj = match ProjectionStore::<InlineProjection<User>>::unique(
            &self.store,
            &UserUniqueKey::Id(user_id),
        )
        .await?
        {
            Some(u) => u,
            None => return Ok(false),
        };
        let user: &User = &user_proj;
        for role_id in &user.roles {
            if self
                .role_has_permission_impl(role_id, permission_code)
                .await?
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub async fn user_can_do(&self, user_id: Ulid, permission_code: &str) -> anyhow::Result<bool> {
        self.user_has_permission(user_id, permission_code).await
    }

    pub async fn user_can_do_any(
        &self,
        user_id: Ulid,
        permission_codes: &[&str],
    ) -> anyhow::Result<bool> {
        for code in permission_codes {
            if self.user_has_permission(user_id, code).await? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub async fn user_can_do_all(
        &self,
        user_id: Ulid,
        permission_codes: &[&str],
    ) -> anyhow::Result<bool> {
        for code in permission_codes {
            if !self.user_has_permission(user_id, code).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub async fn role_has_permission(
        &self,
        role_id: Ulid,
        permission_code: &str,
    ) -> anyhow::Result<bool> {
        self.role_has_permission_impl(&role_id, permission_code)
            .await
    }

    async fn role_has_permission_impl(
        &self,
        role_id: &Ulid,
        permission_code: &str,
    ) -> anyhow::Result<bool> {
        let role_proj = match ProjectionStore::<InlineProjection<Role>>::unique(
            &self.store,
            &RoleUniqueKey::Id(*role_id),
        )
        .await?
        {
            Some(r) => r,
            None => return Ok(false),
        };
        let role: &Role = &role_proj;
        if role.permissions.contains(permission_code) {
            return Ok(true);
        }
        if let Some(parent_id) = role.parent {
            return self
                .role_has_permission_impl(&parent_id, permission_code)
                .await;
        }
        Ok(false)
    }
}
