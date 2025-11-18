use allay_plugin::manager::Plugin;
use allay_plugin::types::{Request, Response};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

struct WorkerThread {
    join_handle: Option<thread::JoinHandle<()>>,
    tx: Sender<Option<(Plugin, Request)>>,
    result_rx: Receiver<Response>,
}

impl WorkerThread {
    fn worker_loop(rx: Receiver<Option<(Plugin, Request)>>, result_tx: Sender<Response>) {
        loop {
            let payload = rx.recv().unwrap();
            if let Some((plugin, request)) = payload {
                let mut plugin = plugin.lock().expect("Failed to lock plugin");
                let response = plugin.handle_request(request).expect("Failed to handle request");
                result_tx.send(response).unwrap();
            } else {
                break;
            }
        }
    }

    fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let (result_tx, result_rx) = std::sync::mpsc::channel();
        let join_handle = thread::spawn(move || {
            Self::worker_loop(rx, result_tx);
        });
        Self {
            join_handle: Some(join_handle),
            tx,
            result_rx,
        }
    }

    fn do_work(&self, plugin: Plugin, request: Request) -> Response {
        assert!(self.join_handle.is_some(), "Worker thread is not running");
        self.tx.send(Some((plugin, request))).unwrap();
        self.result_rx.recv().unwrap()
    }

    fn stop(&mut self) {
        self.tx.send(None).unwrap();
        let join_handle = self.join_handle.take().unwrap();
        join_handle.join().unwrap();
    }
}

impl Drop for WorkerThread {
    fn drop(&mut self) {
        self.stop();
    }
}

pub struct PluginWorker {
    workers: Vec<Mutex<WorkerThread>>,
    idx: AtomicUsize,
}

impl PluginWorker {
    pub fn new(num_workers: usize) -> Self {
        let workers = (0..num_workers).map(|_| Mutex::new(WorkerThread::new())).collect();
        Self {
            workers,
            idx: AtomicUsize::new(0),
        }
    }

    pub fn handle_request(&self, plugin: Plugin, request: Request) -> Response {
        let idx = self.idx.fetch_add(1, Ordering::SeqCst) % self.workers.len();
        let worker = self.workers[idx].lock().unwrap();
        worker.do_work(plugin, request)
    }
}
