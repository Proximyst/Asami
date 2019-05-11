use serenity::model::id::UserId;
use std::collections::HashSet;
use typemap::Key as TypeMapKey;

pub struct OwnerContainer;

impl TypeMapKey for OwnerContainer {
    type Value = HashSet<UserId>;
}
