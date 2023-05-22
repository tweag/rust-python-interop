use std::{future::Future, pin::Pin, task::Poll};

use async_std::task;
use tokio::time::Duration;
use tokio_stream::Stream;

pub(crate) struct FibonacciIterator {
    curr: usize,
    next: usize,
    sleep: Pin<Box<dyn std::future::Future<Output = ()> + Send + Sync>>,
}

impl FibonacciIterator {
    pub(crate) fn new() -> Self {
        Self {
            curr: 0,
            next: 1,
            sleep: Box::pin(task::sleep(Duration::from_millis(100))),
        }
    }
}

impl Iterator for FibonacciIterator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.curr;
        self.curr = self.next;
        self.next += result;

        Some(result)
    }
}

impl Stream for FibonacciIterator {
    type Item = usize;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        match Pin::new(&mut self.sleep).poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(_) => {
                self.sleep = Box::pin(task::sleep(Duration::from_millis(50)));

                let result = self.curr;
                self.curr = self.next;
                self.next += result;

                Poll::Ready(Some(result))
            }
        }
    }
}
