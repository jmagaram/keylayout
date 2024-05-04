use std::sync::{Arc, Mutex};

pub struct VecThreads<T>(Arc<Mutex<Vec<T>>>);

impl<T> VecThreads<T> {
    pub fn new() -> VecThreads<T> {
        VecThreads(Arc::new(Mutex::new(vec![])))
    }

    pub fn clone(&self) -> VecThreads<T> {
        VecThreads(self.0.clone())
    }

    pub fn items(&self) -> Vec<T>
    where
        T: Clone,
    {
        self.0.lock().unwrap().to_vec()
    }

    pub fn push(&mut self, keyboard: T) {
        let mut data = self.0.lock().unwrap();
        data.push(keyboard);
    }

    pub fn push_get_len(&mut self, keyboard: T) -> usize {
        let mut data = self.0.lock().unwrap();
        data.push(keyboard);
        data.len()
    }

    pub fn pop(&mut self) -> Option<T> {
        let mut data = self.0.lock().unwrap();
        data.pop()
    }
}
