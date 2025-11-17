// Entity definitions for migration workflow example
//
// This demonstrates the typical structure of a Toasty entity crate
// All models are defined here for easy tracking and migration generation

use toasty::stmt::Id;

/// User model with authentication fields
#[derive(Debug, toasty::Model)]
pub struct User {
    #[key]
    #[auto]
    pub id: Id<Self>,

    pub name: String,

    #[unique]
    pub username: String,

    #[unique]
    pub email: String,

    #[has_many]
    pub posts: toasty::HasMany<Post>,

    #[has_many]
    pub user_roles: toasty::HasMany<UserRole>,

    pub password: String,

    pub dummy_field: String,
}

/// Blog post model
#[derive(Debug, toasty::Model)]
pub struct Post {
    #[key]
    #[auto]
    pub id: Id<Self>,

    #[index]
    pub user_id: Id<User>,

    #[belongs_to(key = user_id, references = id)]
    pub user: toasty::BelongsTo<User>,

    pub title: String,

    pub content: String,
}

/// Role model for RBAC
#[derive(Debug, toasty::Model)]
pub struct Role {
    #[key]
    #[auto]
    pub id: Id<Self>,

    #[unique]
    pub name: String,

    pub description: Option<String>,

    #[has_many]
    pub user_roles: toasty::HasMany<UserRole>,
}

/// User-Role junction table (many-to-many)
#[derive(Debug, toasty::Model)]
pub struct UserRole {
    #[key]
    #[auto]
    pub id: Id<Self>,

    #[index]
    pub user_id: Id<User>,

    #[index]
    pub role_id: Id<Role>,

    #[belongs_to(key = user_id, references = id)]
    pub user: toasty::BelongsTo<User>,

    #[belongs_to(key = role_id, references = id)]
    pub role: toasty::BelongsTo<Role>,
}
