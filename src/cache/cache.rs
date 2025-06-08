use crate::cache::error::CacheError;

#[cfg_attr(test, mockall::automock)]
pub trait Cache<T: Send + Sync>: Send + Sync {
    fn get_if_present(&self, embedding: &[f32]) -> Result<Option<T>, CacheError>;
    fn insert(&self, embedding: Vec<f32>, response: T) -> Result<(), CacheError>;
    fn try_update(&self, embedding: &[f32], response: T) -> Result<bool, CacheError>;
}
