pub struct CreatePost {
    pub title: String,
    pub content: String,
}

pub struct UpdatePost {
    pub title: Option<String>,
    pub content: Option<String>,
}
