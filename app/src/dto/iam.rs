pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub phc: String,
}

pub struct UpdateUser {
    pub email: Option<String>,
    pub phc: Option<String>,
}

pub struct RegisterInput {
    pub username: String,
    pub email: String,
    pub password: String,
}

pub struct LoginInput {
    pub email: String,
    pub password: String,
}

pub struct GrantRolesInput {
    pub roles: Vec<String>,
}

pub struct RevokeRolesInput {
    pub roles: Vec<String>,
}

pub struct CreateRole {
    pub name: String,
    pub description: Option<String>,
}

pub struct UpdateRole {
    pub name: Option<String>,
    pub description: Option<String>,
}

pub struct GrantPermissionsInput {
    pub permissions: Vec<String>,
}

pub struct RevokePermissionsInput {
    pub permissions: Vec<String>,
}

pub struct CreatePermission {
    pub code: String,
    pub description: Option<String>,
}

pub struct UpdatePermission {
    pub code: Option<String>,
    pub description: Option<String>,
}
