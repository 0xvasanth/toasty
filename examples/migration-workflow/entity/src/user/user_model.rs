use toasty::stmt::Id;

/// User model - handles user authentication and profile
#[derive(Debug, toasty::Model)]
pub struct User {
    /// Primary key
    #[key]
    #[auto]
    pub id: Id<Self>,

    /// User's full name
    pub name: String,

    /// Unique username for login
    #[unique]
    pub username: String,

    /// Unique email address
    #[unique]
    pub email: String,

    /// User's blog posts
    #[has_many]
    pub posts: toasty::HasMany<crate::blog::Post>,

    /// User's role assignments
    #[has_many]
    pub user_roles: toasty::HasMany<crate::rbac::UserRole>,
}
