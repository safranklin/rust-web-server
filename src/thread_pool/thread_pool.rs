use std::fmt;
use std::thread;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>
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

        self.sender.send(Message::NewJob(job)).unwrap();
    }
    
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Asking workers to terminate.");
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers.");
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }

        }
    }
}

// Define Job to be a box of memory with the same trait bounds as the execute function.
type Job = Box<dyn FnOnce() + Send + 'static>;
enum Message {
    NewJob(Job),
    Terminate,
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}
impl Worker {
    fn new(id: usize, reciever: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            // Retrieve the message by locking the reciever (preventing other threads from accessing)
            // unwrap it to panic on any errors (an example may be a posioned mutex which happens
            // if a thread panics before releasing the lock).
            let message = reciever.lock() // We are using a Mutex here to ensure that only a single thread is waiting to recieve a job.
                                          // Any other threads that hit the lock will wait till the lock is released before trying to call
                                          // recv().
                                  .expect("Thread is poisioned. Likely a panic occurred and the lock was not released")
                                  .recv() // If we get the lock call recv to recieve the job from the channel.
                                          // recv will block the thread execution until a message is sent (job is available).
                                  .unwrap();
            match message {
                Message::NewJob(job) => {
                    println!("Worker {} got a job; executing...", id);
                    job();
                }
                Message::Terminate => {
                    println!("Worker {} was asked to terminate.", id);
                    break;
                }
            }
        });
        Worker {
            id,
            thread: Some(thread)
        }
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
    use std::time::{Duration};

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


    #[test]
    fn test_threadpool_basic_sum() {
        let tp_size: i32 = 42;
        // Setup the thread pool
        let tp = ThreadPool::new(tp_size as usize).expect("Failed to create threads");
        let (tx, rx) = mpsc::channel();
        for i in 1..tp_size {
            let tx = tx.clone();
            tp.execute(move || {
                tx.send(i).expect("channel will be waiting");
            });
        }
        drop(tx);
        let result = rx.iter().fold(1, |accumulator, element| accumulator + element);
        assert_eq!(result, (1 .. tp_size).fold(1, |a, b| a + b));
    }

    #[test]
    fn test_threadpool_long_tasks() {
        // For this test we will spawn threads with ids 1 to tp_size
        // Then each thread will send numbers between 1 and 2*tp_size. All threads that are even
        // will have to wait for 3 seconds after being spawned to send their id. This means when
        // we collect results on the reciever that all the odd elements will appear before all
        // the even results suggestting that the thread pools queuing mechanism is working.
        let tp_size: u64 = 5;
        // Setup the thread pool with tp_size number of threads.
        let tp = ThreadPool::new(tp_size as usize).expect("Failed to create threads.");
        // Create a channel to communicate between the main thread and threadpool worker threads.
        let (tx, rx) = mpsc::channel();
        for i in 1..tp_size*2 {
            let tx = tx.clone();
            tp.execute(move || {
                if i % 2 == 0 {
                    // All even threads will need to wait 3 seconds.
                    thread::sleep(Duration::from_secs(3));
                }
                tx.send(i).expect("Channel transmitter should've sent a value.");
            });
        }
        drop(tx);
        let results: Vec<u64> = rx.iter().collect();
        let midpoint = (results.len() as u64) / 2;
        for (index, result) in results.iter().enumerate() {
            if index as u64 <= midpoint {
                // Make sure that all the elements in the first half
                // of the vector are odd.
                assert!(result % 2 != 0);
            } else {
                // Adnt hat all elements after the first half are even.
                assert!(result % 2 == 0);
            }
        }
    }
}