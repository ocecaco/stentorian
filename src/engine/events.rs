use std::sync::mpsc::Sender;
use std::sync::Mutex;

pub trait EventSender<T> {
    fn send(&self, event: T);
}

pub struct ConvertSender<T> {
    sender: Mutex<Option<Sender<T>>>,
}

impl<T> ConvertSender<T> {
    pub fn new(sender: Sender<T>) -> Self {
        ConvertSender {
            sender: Mutex::new(Some(sender)),
        }
    }
}

impl<T: From<U>, U> EventSender<U> for ConvertSender<T> {
    fn send(&self, event: U) {
        let mut sender = self.sender.lock().unwrap();

        let result = if let Some(ref e) = *sender {
            Some(e.send(event.into()))
        } else {
            None
        };

        if let Some(r) = result {
            if r.is_err() {
                *sender = None;
            }
        }
    }
}
