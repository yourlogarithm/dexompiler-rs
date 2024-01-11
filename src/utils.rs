use std::sync::Mutex;

use serde::{Serialize, Serializer};

pub type Error = Box<dyn std::error::Error>;

pub struct MutexWrapper<T: ?Sized>(pub Mutex<T>);

impl<T: ?Sized + Serialize> Serialize for MutexWrapper<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0
            .lock()
            .expect("mutex is poisoned")
            .serialize(serializer)
    }
}