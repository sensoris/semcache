use crate::cache::error::CacheError;

#[cfg_attr(test, mockall::automock)]
pub trait Cache<T: Send + Sync>: Send + Sync {
    fn get_if_present(&self, embedding: &Vec<f32>) -> Result<Option<T>, CacheError>;
    fn put(&self, embedding: Vec<f32>, response: T) -> Result<(), CacheError>;
}
