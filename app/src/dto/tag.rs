pub struct CreateTag {
    pub name: String,
    pub hero: Option<String>,
    pub description: String,
}

pub struct UpdateTag {
    pub name: Option<String>,
    pub hero: Option<String>,
    pub description: Option<String>,
}
