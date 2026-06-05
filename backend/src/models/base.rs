use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Every database-backed model implements this trait.
/// Provides uniform access to the two fields every table row has.
pub trait Entity {
    fn id(&self) -> Uuid;
    fn created_at(&self) -> DateTime<Utc>;
}

/// Implement Entity for any struct that has `id: Uuid` and `created_at: DateTime<Utc>`.
/// Usage:  impl_entity!(User);
#[macro_export]
macro_rules! impl_entity {
    ($t:ty) => {
        impl $crate::models::base::Entity for $t {
            fn id(&self) -> uuid::Uuid {
                self.id
            }
            fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
                self.created_at
            }
        }
    };
}
