use core::{
    pin::Pin,
    task::{Context, Poll},
};

use crossbeam_queue::ArrayQueue;
use futures_util::{stream::Stream, task::AtomicWaker};
use spin::once::Once;

use crate::dbg_println;

const QUEUE_SIZE: usize = 100;

static SCANCODE_STREAM_WAKER: AtomicWaker = AtomicWaker::new();

static SCANCODE_QUEUE: Once<ArrayQueue<u8>> = Once::new();

pub struct ScancodeStream {
    _private: (), // To prevent construction of the struct from outside of the module
}

impl ScancodeStream {
    /// # Panics
    ///
    /// Only single `ScancodeStream` instance can be created.
    #[expect(clippy::new_without_default)]
    #[must_use]
    pub fn new() -> Self {
        if SCANCODE_QUEUE.is_completed() {
            panic!("ScancodeStream::new should only be called once");
        } else {
            SCANCODE_QUEUE.call_once(|| ArrayQueue::new(QUEUE_SIZE));
        }

        Self { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE.get().expect("not initialized");

        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        SCANCODE_STREAM_WAKER.register(cx.waker());

        queue.pop().map_or(Poll::Pending, |scancode| {
            SCANCODE_STREAM_WAKER.take();
            Poll::Ready(Some(scancode))
        })
    }
}

/// Called by the keyboard interrupt handler.
///
/// Must not block or allocate.
pub(super) fn add_scancode(scancode: u8) {
    if let Some(queue) = SCANCODE_QUEUE.get() {
        match queue.push(scancode) {
            Ok(()) => SCANCODE_STREAM_WAKER.wake(),
            Err(_) => dbg_println!("WARNING: scancode queue full; dropping keyboard input"),
        }
    } else {
        dbg_println!("WARNING: scancode queue uninitialized");
    }
}
