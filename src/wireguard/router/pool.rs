use std::mem;
use std::sync::Arc;

use arraydeque::ArrayDeque;
use spin::{Mutex, MutexGuard};
use crossbeam_channel::Receiver;

use super::runq::{RunQueue, ToKey};

const INORDER_QUEUE_SIZE: usize = 64;

pub struct InnerJob<P, B> {
    // peer (used by worker to schedule/handle inorder queue),
    // when the peer is None, the job is complete
    peer: Option<P>,
    pub body: B,
}

pub struct Job<P, B> {
    inner: Arc<Mutex<InnerJob<P, B>>>,
}

impl<P, B> Clone for Job<P, B> {
    fn clone(&self) -> Job<P, B> {
        Job {
            inner: self.inner.clone(),
        }
    }
}

impl<P, B> Job<P, B> {
    pub fn new(peer: P, body: B) -> Job<P, B> {
        Job {
            inner: Arc::new(Mutex::new(InnerJob {
                peer: Some(peer),
                body,
            })),
        }
    }
}

impl<P, B> Job<P, B> {
    /// Returns a mutex guard to the inner job if complete
    pub fn complete(&self) -> Option<MutexGuard<InnerJob<P, B>>> {
        self.inner
            .try_lock()
            .and_then(|m| if m.peer.is_none() { Some(m) } else { None })
    }
}

pub struct InorderQueue<P, B> {
    queue: Mutex<ArrayDeque<[Job<P, B>; INORDER_QUEUE_SIZE]>>,
}

impl<P, B> InorderQueue<P, B> {
    pub fn send(&self, job: Job<P, B>) -> bool {
        self.queue.lock().push_back(job).is_ok()
    }

    pub fn new() -> InorderQueue<P, B> {
        InorderQueue {
            queue: Mutex::new(ArrayDeque::new()),
        }
    }

    #[inline(always)]
    pub fn handle<F: Fn(&mut B)>(&self, f: F) {
        // take the mutex
        let mut queue = self.queue.lock();

        loop {
            // attempt to extract front element
            let front = queue.pop_front();
            let elem = match front {
                Some(elem) => elem,
                _ => {
                    return;
                }
            };

            // apply function if job complete
            let ret = if let Some(mut guard) = elem.complete() {
                mem::drop(queue);
                f(&mut guard.body);
                queue = self.queue.lock();
                false
            } else {
                true
            };

            // job not complete yet, return job to front
            if ret {
                queue.push_front(elem).unwrap();
                return;
            }
        }
    }
}

/// Allows easy construction of a semi-parallel worker.
/// Applicable for both decryption and encryption workers.
#[inline(always)]
pub fn worker_parallel<
    P: ToKey, // represents a peer (atomic reference counted pointer)
    B,        // inner body type (message buffer, key material, ...)
    D,        // device
    W: Fn(&P, &mut B),
    Q: Fn(&D) -> &RunQueue<P>,
>(
    device: D,
    queue: Q,
    receiver: Receiver<Job<P, B>>,
    work: W,
) {
    log::trace!("router worker started");
    loop {
        // handle new job
        let peer = {
            // get next job
            let job = match receiver.recv() {
                Ok(job) => job,
                _ => return,
            };

            // lock the job
            let mut job = job.inner.lock();

            // take the peer from the job
            let peer = job.peer.take().unwrap();

            // process job
            work(&peer, &mut job.body);
            peer
        };

        // process inorder jobs for peer
        queue(&device).insert(peer);
    }
}
