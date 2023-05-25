/*
Copyright (c) 2023 Tam Vu <tamvu@tdt.asia>

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
 */

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

    /// Get a new publisher
    /// This will create new pub/sub and add the sub into the subscriber
    pub fn get_publisher(&mut self) -> Pub<T> {
        let (tx, rx) = spsc_queue::new::<T>(self.capacity);
        self.subs.subscribers.push(rx);
        return tx;
    }

    /// Borrow the subscriber
    pub fn get_borrowed_subscriber(&self) -> &Subscriber<T> {
        return &self.subs;
    }
}

pub struct Subscriber<T: Send + Sync> {
    subscribers: Vec<Sub<T>>
}

impl <T: Send + Sync> Subscriber<T> {
    /// Batch recv with handler
    pub fn batch_recv<F: FnMut(T)>(&self, handler: &mut F) {
        for sub in self.subscribers.as_slice() {
            sub.batch_recv(handler);
        }
    }
}