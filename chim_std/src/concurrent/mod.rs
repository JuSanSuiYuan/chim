//! 并发模块 - Actor 运行时支持

pub mod actor;
pub mod channel;
pub mod distributed_actor;

pub use actor::Actor;
pub use channel::Channel;
pub use distributed_actor::{
    DistributedActor, DistributedActorRef, DistributedActorRuntime,
    NodeId, ActorId, MessageId, RemoteMessage, RemoteSender,
    Connection, Cluster, EventBus, Subscriber, Event, EventId,
    ConsensusModule, ConsensusAlgorithm, Raft, Paxos, Gossip,
    MigrationManager, Migration, MigrationStatus,
    RuntimeConfig, ClusterConfig, ConsensusConfig, MigrationConfig,
};
