use crate::{Error, Result};
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    pub fn build(size: usize) -> Result<Self> {
        if size == 0 {
            return Err(Error::InvalidPoolSize);
        }

        let (sender, receiver) = mpsc::channel();

        // Beause we have a single consumer
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        Ok(Self {
            workers,
            sender: Some(sender),
        })
    }
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender
            .as_ref()
            .expect("No sender")
            .send(job)
            .expect("Could not send job");
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            // println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().expect("Could not join thread")
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            // NOTE: here the thread is locked until I receive a message
            // If i have 4 threads, all 4 threads will be in waiting
            let message = receiver
                .lock()
                .expect("Worker {id} failed to acquire lock")
                .recv();

            // NOTE: the lock is released here, allowing other workers to receive jobs
            // NOTE: because we use mpsc, only one receiver will actually process the job

            match message {
                Ok(job) => {
                    // println!("Worker {id} got a job; executing.");

                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;
