
pub mod spsc_queue;

use std::collections::BinaryHeap;
use std::rc::{Rc, self};
use std::thread::{self, JoinHandle};
use std::time::{Instant, UNIX_EPOCH, SystemTime, Duration};

use crate::spsc_queue::{Pub, Sub};

#[derive(Clone, Copy, Debug, Default)]
struct Test {
    val: i64
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
struct TestLatency {
    time: u128
}

fn test_spsc() {
    const N: i64 = 200_000_000;
    const CAP: usize = 1024 * 1024;

    println!("Total msg: {} - Total publisher: {} - Size per buffer: {}", N, 1, CAP);

    let (tx, rx) = spsc_queue::new::<Test>(CAP);
    let p = thread::spawn(move || {
        let now = Instant::now();
        for i in 0..N {
            tx.push(Test { val: i });
        }
        let elapsed = now.elapsed();
        println!("Push elapsed: {:?}", elapsed);
    });

    let count: Rc<i64> = Rc::new(0);
    let tmp = count.clone();
    let mut handle = { move |ret: Test| {
        // println!("ret: {:?}", ret.val);
        let ptr = Rc::as_ptr(&tmp) as *mut i64;
        unsafe {
            *ptr += 1;
        }
    }};

    let now = Instant::now();

    loop {
        rx.batch_recv(&mut handle);
        if *count.as_ref() == N {
            break;
        }
    }
    let elapsed = now.elapsed();
    println!("Recv elapsed: {:?}", elapsed);
    p.join().expect("Unable to join");
}

fn test_mpsc() {
    const N: i64 = 800_000_000;
    const CAP: usize = 1024 * 1024;
    const NUM_PUB: i64 = 4;
    const INTERVAL: i64 = N / NUM_PUB;

    println!("Total msg: {} - Total publisher: {} - Size per buffer: {}", N, NUM_PUB, CAP);

    let mut publishers = Vec::<Pub<Test>>::new();
    let mut receivers = Vec::<Sub<Test>>::new();
    let mut thread_handles = Vec::<JoinHandle<()>>::new();

    for _i in 0..NUM_PUB {
        let (tx, rx) = spsc_queue::new::<Test>(CAP);
        publishers.push(tx);
        receivers.push(rx);
    }

    for i in 0..NUM_PUB {
        let tx = publishers.remove(0);
        let handle = thread::spawn(move || {
            for i in (i*INTERVAL)..((i + 1) * INTERVAL) {
                tx.push(Test { val: i })
            }
        });
        thread_handles.push(handle);
    }

    let count: Rc<i64> = Rc::new(0);
    let tmp = count.clone();
    let mut handle = { move |ret: Test| {
        // println!("ret: {:?}", ret.val);
        let ptr = Rc::as_ptr(&tmp) as *mut i64;
        unsafe {
            *ptr += 1;
        }
    }};

    let now = Instant::now();
    loop {
        let slice = receivers.as_slice();
        for recveiver in slice {
            recveiver.batch_recv(&mut handle);
        }

        if *count.as_ref() == N {
            break;
        }
    }
    let elapsed = now.elapsed();
    println!("Recv elapsed: {:?}", elapsed);

    for t in thread_handles {
        t.join().expect("Unable to join thread");
    }

}

fn test_latency() {
    const N: i64 = 100;
    const CAP: usize = 1024 * 1024;
    const NUM_PUB: i64 = 1;
    const INTERVAL: i64 = N / NUM_PUB;

    let mut publishers = Vec::<Pub<TestLatency>>::new();
    let mut receivers = Vec::<Sub<TestLatency>>::new();
    let mut thread_handles = Vec::<JoinHandle<()>>::new();

    let now = Instant::now();

    for _i in 0..NUM_PUB {
        let (tx, rx) = spsc_queue::new::<TestLatency>(CAP);
        publishers.push(tx);
        receivers.push(rx);
    }

    for i in 0..NUM_PUB {
        let tx = publishers.remove(0);
        let epoch = now.clone();
        let handle = thread::spawn(move || {
            for _j in (i*INTERVAL)..((i + 1) * INTERVAL) {
                let nano = Instant::now().duration_since(epoch).as_nanos();
                tx.push(TestLatency { time: nano });
                thread::sleep(Duration::new(0, 100_000_000));
            }
        });
        thread_handles.push(handle);
    }

    let count: Rc<i64> = Rc::new(0);
    let tmp = count.clone();
    let total_latency_count: Rc<u128> = Rc::new(0);
    let tmp_total_latency_count = total_latency_count.clone();
    let total_latency: Rc<u128> = Rc::new(0);
    let tmp_total_latency: Rc<u128> = total_latency.clone();

    let tmp_epoch = now.clone();
    let mut handle = { move |ret: TestLatency| {
        let nano = Instant::now().duration_since(tmp_epoch).as_nanos();
        let ptr = Rc::as_ptr(&tmp) as *mut i64;
        let ptr_total_latency = Rc::as_ptr(&tmp_total_latency) as *mut u128;
        let ptr_total_latency_count = Rc::as_ptr(&tmp_total_latency_count) as *mut u128;
        unsafe {
            *ptr += 1;
            if nano != 0 {
                *ptr_total_latency += (nano - ret.time);
                *ptr_total_latency_count += 1;
            }
        }

    }};

    let now = Instant::now();
    loop {
        let slice = receivers.as_slice();
        for recveiver in slice {
            recveiver.batch_recv(&mut handle);
        }

        if *count.as_ref() == N {
            break;
        }
    }
    let elapsed = now.elapsed();
    println!("Recv elapsed: {:?}", elapsed);


    println!("Total latency: {:?} - count: {:?} - avg: {:?}", total_latency.as_ref(), total_latency_count.as_ref(), total_latency.as_ref() / total_latency_count.as_ref());

    for t in thread_handles {
        t.join().expect("Unable to join thread");
    }
}

fn test_sort() {
    const N: i64 = 100;
    const CAP: usize = 8;
    const NUM_PUB: i64 = 1;
    const INTERVAL: i64 = N / NUM_PUB;

    let mut publishers = Vec::<Pub<TestLatency>>::new();
    let mut receivers = Vec::<Sub<TestLatency>>::new();
    let mut thread_handles = Vec::<JoinHandle<()>>::new();

    let now = Instant::now();

    for _i in 0..NUM_PUB {
        let (tx, rx) = spsc_queue::new::<TestLatency>(CAP);
        publishers.push(tx);
        receivers.push(rx);
    }

    for i in 0..NUM_PUB {
        let tx = publishers.remove(0);
        let epoch = now.clone();
        let handle = thread::spawn(move || {
            for i in (i*INTERVAL)..((i + 1) * INTERVAL) {
                let nano = Instant::now().duration_since(epoch).as_nanos();
                tx.push(TestLatency { time: nano });
            }
        });
        thread_handles.push(handle);
    }

    let count: Rc<i64> = Rc::new(0);
    let tmp = count.clone();

    let heap = Rc::new(BinaryHeap::<TestLatency>::new());
    let tmp_heap = heap.clone();
    let mut handle = { move |ret: TestLatency| {
        
        let ptr = Rc::as_ptr(&tmp) as *mut i64;
        let heap_ptr = Rc::as_ptr(&tmp_heap) as *mut BinaryHeap<TestLatency>;
        unsafe {
            *ptr += 1;
            (*heap_ptr).push(ret);
        }

    }};

    let now = Instant::now();
    loop {
        let slice = receivers.as_slice();
        for recveiver in slice {
            recveiver.batch_recv(&mut handle);
        }

        let h = heap.as_ref().clone();
        let vec = h.into_sorted_vec();     
        println!("Vec: {:?}", vec); 
        let tmp_ptr = Rc::as_ptr(&heap) as *mut BinaryHeap<TestLatency>;
        unsafe {
            (*tmp_ptr).clear();
        }

        if *count.as_ref() == N {
            break;
        }
    }
    let elapsed = now.elapsed();
    println!("Recv elapsed: {:?}", elapsed);

    for t in thread_handles {
        t.join().expect("Unable to join thread");
    }
}

fn main() {
    for _i in 0..10 {
        test_mpsc()
    }
}
