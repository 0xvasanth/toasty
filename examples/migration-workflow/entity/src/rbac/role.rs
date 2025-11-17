use toasty::stmt::Id;

/// Role model for RBAC (Role-Based Access Control)
#[derive(Debug, toasty::Model)]
pub struct Role {
    /// Primary key
    #[key]
    #[auto]
    pub id: Id<Self>,

    /// Unique role name (e.g., "admin", "user", "moderator")
    #[unique]
    pub name: String,

    /// Role description
    pub description: Option<String>,

    /// Users assigned to this role
    #[has_many]
    pub user_roles: toasty::HasMany<crate::rbac::UserRole>,
}
