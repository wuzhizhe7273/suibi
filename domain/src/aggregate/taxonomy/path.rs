use std::path::PathBuf;

use anyhow::anyhow;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct TaxonomyPath(PathBuf);

impl PartialEq for TaxonomyPath {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for TaxonomyPath {}

impl TaxonomyPath {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        TaxonomyPath(path.into())
    }

    pub fn into_inner(self) -> PathBuf {
        self.0
    }

    pub fn as_path(&self) -> &std::path::Path {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.as_os_str().is_empty()
    }

    pub fn parent(&self) -> Option<TaxonomyPath> {
        self.0.parent().map(|p| TaxonomyPath(p.to_path_buf()))
    }

    pub fn ancestors(&self) -> impl Iterator<Item = TaxonomyPath> {
        self.0
            .ancestors()
            .skip(1)
            .map(|p| TaxonomyPath(p.to_path_buf()))
    }

    pub fn is_parent_of(&self, other: &TaxonomyPath) -> bool {
        match self.parent() {
            Some(parent) => parent == *other,
            None => false,
        }
    }

    pub fn is_ancestor_of(&self, other: &TaxonomyPath) -> bool {
        other.ancestors().any(|ancestor| ancestor == *self)
    }

    pub fn is_child_of(&self, other: &TaxonomyPath) -> bool {
        other.is_parent_of(self)
    }

    pub fn is_descendant_of(&self, other: &TaxonomyPath) -> bool {
        other.is_ancestor_of(self)
    }
}

impl TryFrom<String> for TaxonomyPath {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(anyhow!("path cannot be empty"));
        }
        Ok(TaxonomyPath::new(value))
    }
}

impl std::fmt::Display for TaxonomyPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}
