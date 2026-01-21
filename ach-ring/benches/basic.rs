use ach_ring::Ring;
use criterion::{criterion_group, criterion_main, Criterion};
use crossbeam_queue::ArrayQueue;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

pub fn mpsc(c: &mut Criterion) {
    const THREAD_NUM: usize = 4;
    const CAPACITY: usize = 4;
    c.bench_function("ring::mpsc", |b| {
        b.iter_custom(|iters| {
            static RING: Ring<u32, CAPACITY> = Ring::new();
            let start = Instant::now();

            crossbeam_utils::thread::scope(|scope| {
                let msgs = iters as usize * THREAD_NUM;

                for _ in 0..THREAD_NUM {
                    scope.spawn(move |_| {
                        for _ in 0..msgs / THREAD_NUM {
                            while  RING.push(Default::default()).is_err() {
                                thread::yield_now();
                            }
                        }
                    });
                }

                for _ in 0..msgs {
                    while RING.pop().is_err() {}
                }
            })
            .unwrap();

            start.elapsed()
        });
    });
    c.bench_function("crossbeam::mpsc", |b| {
        b.iter_custom(|iters| {
            let queue = Arc::new(ArrayQueue::<u32>::new(CAPACITY));
            let start = Instant::now();

            crossbeam_utils::thread::scope(|scope| {
                let msgs = iters as usize * THREAD_NUM;

                for _ in 0..THREAD_NUM {
                    let tx = queue.clone();
                    scope.spawn(move |_| {
                        for _ in 0..msgs / THREAD_NUM {
                            while tx.push(Default::default()).is_err() {
                                thread::yield_now();
                            }
                        }
                    });
                }

                for _ in 0..msgs {
                    while queue.pop().is_none() {}
                }
            })
            .unwrap();

            start.elapsed()
        });
    });
    c.bench_function("flume::mpsc", |b| {
        b.iter_custom(|iters| {
            let (tx, rx) = flume::bounded::<u32>(CAPACITY);
            let start = Instant::now();

            crossbeam_utils::thread::scope(|scope| {
                let msgs = iters as usize * THREAD_NUM;

                for _ in 0..THREAD_NUM {
                    let tx = tx.clone();
                    scope.spawn(move |_| {
                        for _ in 0..msgs / THREAD_NUM {
                            tx.send(Default::default()).unwrap()
                        }
                    });
                }

                for _ in 0..msgs {
                    rx.recv().unwrap();
                }
            })
            .unwrap();

            start.elapsed()
        });
    });
}

criterion_group!(benches, mpsc);
criterion_main!(benches);
