//! 通道模块 - 占位符

use std::sync::mpsc;

/// 通道发送端
pub struct Sender<T> {
    inner: mpsc::Sender<T>,
}

impl<T> Sender<T> {
    pub fn send(&self, value: T) -> Result<(), mpsc::SendError<T>> {
        self.inner.send(value)
    }
}

/// 通道接收端
pub struct Receiver<T> {
    inner: mpsc::Receiver<T>,
}

impl<T> Receiver<T> {
    pub fn recv(&self) -> Result<T, mpsc::RecvError> {
        self.inner.recv()
    }
    
    pub fn try_recv(&self) -> Result<T, mpsc::TryRecvError> {
        self.inner.try_recv()
    }
}

/// 通道
pub struct Channel<T> {
    sender: Sender<T>,
    receiver: Receiver<T>,
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            sender: Sender { inner: tx },
            receiver: Receiver { inner: rx },
        }
    }
    
    pub fn sender(&self) -> &Sender<T> {
        &self.sender
    }
    
    pub fn receiver(&self) -> &Receiver<T> {
        &self.receiver
    }
}

impl<T> Default for Channel<T> {
    fn default() -> Self {
        Self::new()
    }
}
