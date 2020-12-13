use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

type Job = Box<dyn FnBox + Send + 'static>;

impl ThreadPool {
    /// ThreadPool を生成する
    ///
    /// # 引数
    /// - size: プールのスレッド数
    ///
    /// # パニック
    /// size が 0 なら `new` はパニックする。
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        // while let Ok(job) = receiver.lock().unwrap().recv().unwrap() にすることもできなくはないけど、
        // while let にするとスコープが変わって job のライフライムが伸びるので、 Worker が次の Job を受けることができなくなってしまうらしい。
        // 詳細: https://doc.rust-jp.rs/book-ja/ch20-02-multithreaded.html の一番下。
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();
            println!("Worker {} got a job; executing.", id);
            job.call_box();
        });

        Worker { id, thread }
    }
}
