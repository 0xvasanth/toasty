// Entity crate - organized by domain
//
// Structure:
// - user/      User-related models
// - blog/      Blog-related models (posts, comments, etc.)
// - rbac/      Role-Based Access Control models

pub mod user;
pub mod blog;
pub mod rbac;

// Re-export all models for convenience
pub use user::User;
pub use blog::Post;
pub use rbac::{Role, UserRole};

// Re-export toasty for use in applications
pub use toasty;
