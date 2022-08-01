use std::thread;
use std::thread::spawn;
use crate::worker::Worker;
pub type Task =Box<dyn FnOnce()+ Send + 'static>;
pub struct ThreadPoolHandle{
    sender:crossbeam_channel::Sender<Task>
}
impl ThreadPoolHandle{
    pub fn new(sender:crossbeam_channel::Sender<Task>) -> ThreadPoolHandle {
        ThreadPoolHandle{
            sender
        }
    }
    pub fn execute<F>(&self, f: F)
        where
            F: FnOnce(),
            F: Send + 'static,//+ Send + 'static,
    {
        self.sender.send(Box::new(f));
    }
}

pub struct ThreadPool{
    threads: Vec<Worker>,
    sender:crossbeam_channel::Sender<Task>
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        let (tx,rx)=crossbeam_channel::unbounded();
        let mut threads =vec![];
        for _ in 0..size{
            threads.push(Worker::new(rx.clone()))
        }
        ThreadPool{
            threads,
            sender: tx
        }
    }
    pub fn get_handle(&self) -> ThreadPoolHandle {
        ThreadPoolHandle::new(self.sender.clone())
    }
}