use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PermissionCode(String);

impl PermissionCode {
    pub fn new(code: impl Into<String>) -> anyhow::Result<Self> {
        let code = code.into();
        Self::validate(&code)?;
        Ok(Self(code))
    }

    fn validate(code: &str) -> anyhow::Result<()> {
        if code.is_empty() {
            return Err(anyhow::anyhow!("code cannot be empty"));
        }
        let parts: Vec<&str> = code.split('.').collect();
        for part in parts {
            if part.is_empty() {
                return Err(anyhow::anyhow!("code cannot contain empty segments"));
            }
            if !part.chars().all(|c| c.is_ascii_alphabetic() || c == '_') {
                return Err(anyhow::anyhow!(
                    "code segments must only contain A-Za-z_: got '{}'",
                    part
                ));
            }
        }
        Ok(())
    }
}

impl std::ops::Deref for PermissionCode {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_code() {
        assert!(PermissionCode::new("user.create").is_ok());
        assert!(PermissionCode::new("user_read").is_ok());
        assert!(PermissionCode::new("ROLE_CREATE").is_ok());
        assert!(PermissionCode::new("a.b.c").is_ok());
    }

    #[test]
    fn test_invalid_code() {
        assert!(PermissionCode::new("").is_err());
        assert!(PermissionCode::new("user.create.123").is_err());
        assert!(PermissionCode::new("user-create").is_err());
        assert!(PermissionCode::new("user..create").is_err());
    }
}
