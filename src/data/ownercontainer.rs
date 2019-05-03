use std::collections::HashSet;
use serenity::model::id::UserId;
use typemap::Key as TypeMapKey;

pub struct OwnerContainer;

impl TypeMapKey for OwnerContainer {
    type Value = HashSet<UserId>;
}
