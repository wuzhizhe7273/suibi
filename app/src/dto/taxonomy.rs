use domain::aggregate::taxonomy::TaxonomyPath;

pub struct CreateTaxonomy {
    pub name: String,
    pub slug: String,
    pub path: TaxonomyPath,
    pub description: String,
    pub hero: Option<String>,
}

pub struct UpdateTaxonomy {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub path: Option<TaxonomyPath>,
    pub description: Option<String>,
    pub hero: Option<String>,
}
