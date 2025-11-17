use toasty::stmt::Id;

/// Blog post model
#[derive(Debug, toasty::Model)]
pub struct Dummy {
    /// Primary key
    #[key]
    #[auto]
    pub id: Id<Self>,

    /// Foreign key to user
    #[index]
    pub user_id: Id<crate::user::User>,

    /// Reference to post author
    #[belongs_to(key = user_id, references = id)]
    pub user: toasty::BelongsTo<crate::user::User>,

    /// Post title
    pub title: String,

    /// Post content
    pub content: String,
}
