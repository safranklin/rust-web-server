use std::fmt;

pub struct ThreadPool;
impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is less than or equal to zero.
    pub fn new(size: usize) -> Result<ThreadPool, PoolCreationError> {
        if size <= 0 {
            return Err(PoolCreationError)
        }

        return Ok(ThreadPool)
    }

    pub fn execute<F>(&self, f: F) where F: FnOnce() + Send + 'static, {
        println!("Would execute function f!");
    }
    
}

// Define our error types. These may be customized for our error handling cases.
// Now we will be able to write our own errors, defer to an underlying error
// implementation, or do something in between.
#[derive(Debug, Clone)]
pub struct PoolCreationError;

impl fmt::Display for PoolCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not create ThreadPool, the size must be greater than zero!")
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acceptable_threadpool_creation() {
        assert!(ThreadPool::new(5).is_ok());
        assert!(ThreadPool::new(1).is_ok());
        assert!(ThreadPool::new(100).is_ok());
    }

    #[test]
    fn test_threadpool_creation_zero_thread_count() {
        assert!(ThreadPool::new(0).is_err());
    }
}