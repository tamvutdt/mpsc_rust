
use std::ops::{DerefMut};
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use crossbeam_utils::CachePadded;

pub fn is_power_of_two(num: usize) -> bool {
    return (num & (num - 1)) == 0;
}

pub fn next_power_of_two(num: usize) -> usize {
    let mut v: usize = num;
    v -= 1;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v |= v >> 32;
    v += 1;
    return v;
}

pub fn new<T>(cap: usize) -> (Pub<T>, Sub<T>) {
    let rb = RingBuffer::<T>::new(cap);
    let arc = Arc::new(rb);
    let arc_sender = arc.clone();

    return (Pub { rb: arc_sender }, Sub { rb: arc });
}

pub struct Pub<T> {
    rb: Arc<RingBuffer<T>>
}

pub struct Sub<T> {
    rb: Arc<RingBuffer<T>>
}

impl<T> Sub<T> {
    pub fn batch_recv<F: FnMut(T)> (&self, handler: &mut F) {
        let raw_ptr = Arc::as_ptr(&self.rb) as *mut RingBuffer<T>;
        unsafe {
            (*raw_ptr).batch_recv(handler);
        }
    }
}

impl<T> Pub<T> {
    pub fn get_raw_ptr(&self) -> *mut RingBuffer<T> {
        return Arc::as_ptr(&self.rb) as *mut RingBuffer<T>;
    } 

    pub fn push(&self, val: T) {
        let raw_ptr = Arc::as_ptr(&self.rb) as *mut RingBuffer<T>;
        unsafe {
            (*raw_ptr).push(val);
        }
    }
}

pub struct RingBuffer<T> {
    slots: Vec<Option<T>>,
    capacity: usize,
    consumer_write_idx_cache: CachePadded<i64>,      // Need atomic
    consumer_read_idx_cache: CachePadded<i64>,       // Don't need atomic caused it's single thread
    producer_write_idx_cache: CachePadded<i64>,      // Don't need atomic caused it's single thread
    producer_read_idx_cache: CachePadded<i64>,       // Need atomic
    write_idx: CachePadded<AtomicI64>,
    read_idx: CachePadded<AtomicI64>
}

impl<T> RingBuffer<T> {
    pub fn new(cap: usize) -> Self {
        let mut capacity = cap;
        if !is_power_of_two(cap) {
            capacity = next_power_of_two(capacity);
        }

        let mut vec = Vec::<Option<T>>::new();
        for _i in 0..capacity {
            vec.push(None);
        }

        return RingBuffer {
            slots: vec,
            capacity: capacity,
            consumer_write_idx_cache: CachePadded::new(0i64),
            consumer_read_idx_cache: CachePadded::new(0i64),
            producer_write_idx_cache: CachePadded::new(0i64),
            producer_read_idx_cache: CachePadded::new(0i64),
            write_idx: CachePadded::new(AtomicI64::new(0)),
            read_idx: CachePadded::new(AtomicI64::new(0))
        };
    }

    fn push(&mut self, val: T) {
        let tmp_write_idx = self.producer_write_idx_cache.into_inner();
        let mut next_write_idx = tmp_write_idx + 1;
        if next_write_idx == self.capacity as i64 {
            next_write_idx = 0;
        }

        while next_write_idx == self.producer_read_idx_cache.into_inner() {
            *self.producer_read_idx_cache.deref_mut() = self.read_idx.load(Ordering::Acquire);
        }

        self.slots[tmp_write_idx as usize].replace(val);
        (*self.producer_write_idx_cache.deref_mut()) = next_write_idx;
        self.write_idx.store(next_write_idx, Ordering::Release);
    }

    fn batch_recv<F: FnMut(T)> (&mut self, handler: &mut F) {
        let available_read = self.get_available_read();
        if available_read == 0 {
            return;
        }

        let mut tmp_idx = self.consumer_read_idx_cache.into_inner();
        for _ in 0..available_read {
            match self.get_at(tmp_idx) {
                Some(msg) => {
                    handler(msg);
                },
                None => {
                    continue;
                }
            }
            
            tmp_idx += 1;
            if tmp_idx == self.capacity as i64 {
                tmp_idx = 0;
            }
        }

        self.set_read_idx(tmp_idx);
    }

    fn get_at(&mut self, idx: i64) -> Option<T> {
        match self.slots.get_mut(idx as usize) {
            Some(slot) => {
                let tmp_ret = slot.take();
                return tmp_ret;
            }, 
            None => {
                return None;
            }
        }
    }

    fn set_read_idx(&mut self, read_idx: i64) {
        let tmp = self.consumer_read_idx_cache.deref_mut();
        *tmp = read_idx;
        self.read_idx.store(read_idx, Ordering::Release);
    }

    fn get_available_read(&self) -> usize {
        let tmp_write_idx = self.write_idx.load(Ordering::Relaxed);
        let mut diff = tmp_write_idx - self.consumer_read_idx_cache.into_inner();
        if diff < 0 {
            diff += self.capacity as i64;
        }
        return diff as usize;
    }
}