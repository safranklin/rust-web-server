use std::fmt;
use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>
}
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

        // We are going to use channels to send a job from the threadpool
        // to the worker threads.
        let (sender, reciever) = mpsc::channel();

        // We are going to share the reciever amongst multiple threads so
        // wrap it in an Atomic Reference Counter and Mutex combo.
        let reciever = Arc::new(Mutex::new(reciever));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&reciever)));
        }

        return Ok(ThreadPool { workers, sender })
    }

    pub fn execute<F>(&self, f: F) where F: FnOnce() + Send + 'static, {
        let job = Box::new(f);

        self.sender.send(job).unwrap();
    }
    
}

// Define Job to be a box of memory with the same trait bounds as the execute function.
type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}
impl Worker {
    fn new(id: usize, reciever: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            // Retrieve the job by locking the reciever (preventing other threads from accessing)
            // unwrap it to panic on any errors (an example may be a posioned mutex which happens
            // if a thread panics before releasing the lock).
            let job = reciever.lock() // We are using a Mutex here to ensure that only a single thread is waiting to recieve a job.
                                      // Any other threads that hit the lock will wait till the lock is released before trying to call
                                      // recv().
                              .expect("Thread is poisioned. Likely a panic occurred and the lock was not released")
                              .recv() // If we get the lock call recv to recieve the job from the channel.
                                      // recv will block the thread execution until a message is sent (job is available).
                              .unwrap();
            println!("Worker {} got a job; executing...", id);
            job();
        });


        Worker {id, thread}
    }
}


// Errors:

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