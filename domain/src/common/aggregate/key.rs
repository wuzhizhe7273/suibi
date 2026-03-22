pub trait UniqueKey {}
pub trait SearchKey {}
impl UniqueKey for () {}
impl SearchKey for () {}
