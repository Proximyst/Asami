use parking_lot::Mutex;
use serenity::client::bridge::gateway::ShardManager;
use std::sync::Arc;
use typemap::Key as TypeMapKey;

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}
