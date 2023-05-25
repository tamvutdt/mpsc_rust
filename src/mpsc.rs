use crate::spsc_queue::{self, Pub, Sub};

pub struct MPSC<T: Send + Sync> {
    capacity: i64,
    subs: Subscriber<T>
}

impl <T: Send + Sync> MPSC<T> {

    pub fn new(cap: i64) -> Self {
        let sub = Subscriber {
            subscribers: Vec::<Sub<T>>::new()       
        };

        return MPSC{ capacity: cap, subs: sub};
    }

    pub fn get_publisher(&mut self) -> Pub<T> {
        // create new spsc
        let (tx, rx) = spsc_queue::new::<T>(self.capacity);
        self.subs.subscribers.push(rx);
        return tx;
    }

    pub fn get_borrowed_subscriber(&self) -> &Subscriber<T> {
        return &self.subs;
    }
}

pub struct Subscriber<T: Send + Sync> {
    subscribers: Vec<Sub<T>>
}

impl <T: Send + Sync> Subscriber<T> {
    pub fn batch_recv<F: FnMut(T)>(&self, handler: &mut F) {
        for sub in self.subscribers.as_slice() {
            sub.batch_recv(handler);
        }
    }
}