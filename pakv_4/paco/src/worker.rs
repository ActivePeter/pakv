use std::thread::{JoinHandle, spawn};
use crate::threadpool::Task;

pub struct Worker{
    thread: JoinHandle<()>,
}
impl Worker{
    pub fn new(recv:crossbeam_channel::Receiver<Task>)->Worker{
        Worker{
            thread:spawn(move ||{
                loop {
                    let r=recv.recv().unwrap();
                    r();
                }
            })
        }
    }
}