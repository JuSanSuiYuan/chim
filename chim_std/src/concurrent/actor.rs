//! Actor 运行时 - 占位符

use std::sync::mpsc;

/// Actor trait
pub trait Actor {
    type Message;
    
    fn receive(&mut self, msg: Self::Message);
}

/// Actor 引用
pub struct ActorRef<M> {
    sender: mpsc::Sender<M>,
}

impl<M> ActorRef<M> {
    pub fn send(&self, msg: M) -> Result<(), mpsc::SendError<M>> {
        self.sender.send(msg)
    }
}

/// Actor 系统
pub struct ActorSystem {
    // 占位符
}

impl ActorSystem {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for ActorSystem {
    fn default() -> Self {
        Self::new()
    }
}
