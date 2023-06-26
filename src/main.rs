use rustc_hash::FxHashMap;

use ordered_float::OrderedFloat;

use std::cell::Cell;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

use std::{thread, time};

use crossbeam_channel::{bounded, Receiver, Sender};
use std::sync::atomic::{AtomicBool, Ordering};

use std::ops::Deref;

#[derive(Clone)]
pub struct runner {
    value: Arc<u8>,
    reader_thread: Arc<AtomicBool>,
    writer_thread: Arc<AtomicBool>,
    asks: side,
    bids: side,
    receiver: Receiver<u8>,
}

impl runner {
    fn new() -> (Self, Sender<u8>) {
        let reader_thread = Arc::new(AtomicBool::new(false));

        let cloned_reader_thread = reader_thread.clone();
        let cloned_two = reader_thread.clone();

        let writer_thread = Arc::new(AtomicBool::new(true));
        let cloned_2 = reader_thread.clone();
        let cloned_2_2 = reader_thread.clone();
        let (rx, tx) = bounded(5);

        (
            runner {
                value: Arc::new(8),
                reader_thread: reader_thread,
                writer_thread: writer_thread,
                asks: side::new(cloned_reader_thread, cloned_2),
                bids: side::new(cloned_two, cloned_2_2),
                receiver: tx,
            },
            rx,
        )
    }
    fn process_depths(&mut self) {
        loop {
            while self.writer_thread.load(Ordering::Relaxed) {
                if let Ok(_) = self.receiver.recv() {
                    self.writer_thread.store(false, Ordering::SeqCst);
                    println!("1 2 WRITER THREAD UPDATING BOOK");
                    self.asks.side.insert(OrderedFloat(10.0), Level::new(10.0));
                    self.reader_thread.store(true, Ordering::SeqCst); // trigger the reader back on
                    continue;
                }
            }
        }
    }
    fn traverse(&mut self) {
        // wait for depth process to release reader_thread
        loop {
            thread::sleep(time::Duration::from_secs(3));
            while self.reader_thread.load(Ordering::Relaxed) {
                self.reader_thread.store(false, Ordering::SeqCst);
                let _ = thread::scope(|s| {
                    let ask_traverser = s.spawn(|| self.asks.traverse_asks());
                    let bid_traverser = s.spawn(|| self.bids.traverse_bids());
                    ask_traverser.join();
                    bid_traverser.join();
                });
                println!("3 3 READER thread producing quote");
                self.writer_thread.store(true, Ordering::SeqCst); // turn on the writer
                break;
            }
        }
    }

    fn unreader_thread(&mut self) {
        let reader_thread = self.reader_thread.clone();
        let t1 = thread::spawn(move || loop {
            thread::sleep(time::Duration::from_secs(5));
            _ = reader_thread.load(Ordering::SeqCst);
        });
        t1.join();
    }
}

#[derive(Copy, Clone, Debug)]
struct Level<LiquidityNode> {
    price: f64,
    deque: [LiquidityNode; 2],
}

#[derive(Copy, Clone)]
pub struct LiquidityNode {
    q: f64,
    l: u8,
}

impl Level<LiquidityNode> {
    fn new(price_level: f64) -> Self {
        Level {
            price: price_level,
            deque: [
                LiquidityNode { q: 0.0, l: 1 },
                LiquidityNode { q: 0.0, l: 2 },
            ],
        }
    }
}

#[derive(Clone)]
pub struct side {
    side: FxHashMap<OrderedFloat<f64>, Level<LiquidityNode>>,
}

fn round(num: f64, place: f64) -> f64 {
    (num * place).round() / (place)
}

#[derive(Clone, Debug, Copy)]
pub struct Deal {}

impl side {
    fn new(reader_thread: Arc<AtomicBool>, internal_reader_thread: Arc<AtomicBool>) -> Self {
        let mut side = FxHashMap::<OrderedFloat<f64>, Level<LiquidityNode>>::default();
        let level = Level::new(40.0);
        side.insert(OrderedFloat(round(40.0, 100.0)), level);

        let mut side = side {
            // side: FxHashMap::<OrderedFloat<f64>, Level<LiquidityNode>>::default(),
            side: side,
        };
        side
    }

    fn traverse_bids(&mut self) {
        println!("3 2 READER THREAD traversing bids");
        thread::sleep(time::Duration::from_millis(1));
        /*
        while let Some(val) = self.side.get_mut().into_iter().next() {
            println!("{:#?}")
        }
        */
    }

    fn traverse_asks(&mut self) {
        println!("3 2 READER THREAD traversing asks");
        thread::sleep(time::Duration::from_millis(1));
    }
}

fn main() {
    let (mut runner, producer) = runner::new();
    let mut reader_runner = runner.clone();

    let sender = thread::spawn(move || loop {
        // thread::sleep(time::Duration::from_millis(3));
        producer.send(8);
    });

    let writer = thread::spawn(move || reader_runner.process_depths());
    let mut writer_runner = runner.clone();
    let runner = thread::spawn(move || writer_runner.traverse());
    sender.join();
    writer.join();
    runner.join();
    println!("ending program")
}
