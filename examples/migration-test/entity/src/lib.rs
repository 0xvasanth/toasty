use toasty::stmt::Id;

#[derive(Debug, toasty::Model)]
pub struct User {
    #[key]
    #[auto]
    pub id: Id<Self>,
    pub name: String,
    #[unique]
    pub email: String,
    pub bio: Option<String>,
    pub age: Option<u32>,
}
