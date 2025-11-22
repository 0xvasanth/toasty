use toasty::stmt::Id;

/// User-Role junction table for many-to-many relationship
#[derive(Debug, toasty::Model)]
pub struct UserRole {
    /// Primary key
    #[key]
    #[auto]
    pub id: Id<Self>,

    /// Foreign key to user
    #[index]
    pub user_id: Id<crate::user::User>,

    /// Foreign key to role
    #[index]
    pub role_id: Id<crate::rbac::Role>,

    /// Reference to user
    #[belongs_to(key = user_id, references = id)]
    pub user: toasty::BelongsTo<crate::user::User>,

    /// Reference to role
    #[belongs_to(key = role_id, references = id)]
    pub role: toasty::BelongsTo<crate::rbac::Role>,
}
