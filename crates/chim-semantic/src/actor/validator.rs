use crate::type_pool::TypePool;
use crate::ChimError;
use chim_span::Span;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Actor {
    pub name: String,
    pub state: Option<ActorState>,
    pub mailbox: MailboxConfig,
    pub handlers: Vec<MessageHandler>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActorState {
    pub fields: Vec<StateField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StateField {
    pub name: String,
    pub ty: crate::TypeId,
    pub is_mutable: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MailboxConfig {
    pub capacity: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MessageHandler {
    pub message_type: String,
    pub params: Vec<HandlerParam>,
    pub body: Vec<crate::ast::Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HandlerParam {
    pub name: String,
    pub ty: crate::TypeId,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub payload: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActorRef {
    pub actor_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActorRuntime {
    pub actors: Vec<Actor>,
    pub channels: Vec<Channel>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Channel {
    pub from: ActorRef,
    pub to: ActorRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    Fifo,
    Lifo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ordering {
    Ordered,
    Unordered,
}

#[derive(Debug)]
pub struct ActorValidator {
    actors: Vec<Actor>,
    messages: Vec<Message>,
    errors: Vec<ChimError>,
}

impl ActorValidator {
    pub fn new() -> Self {
        ActorValidator {
            actors: Vec::new(),
            messages: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn validate(&mut self, _pool: &TypePool, _program: &crate::ast::Program) -> Result<(), Vec<ChimError>> {
        Ok(())
    }

    pub fn add_actor(&mut self, actor: Actor) {
        self.actors.push(actor);
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actor_creation() {
        let pool = TypePool::new();
        let mut validator = ActorValidator::new();

        let actor = Actor {
            name: "Worker".to_string(),
            state: None,
            mailbox: MailboxConfig { capacity: Some(100) },
            handlers: Vec::new(),
            span: Span::new(chim_span::FileId(0), 0, 0, 0, 0),
        };

        validator.add_actor(actor);
        assert_eq!(validator.actors.len(), 1);
    }
}
