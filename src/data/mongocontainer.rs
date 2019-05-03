use r2d2::Pool;
use r2d2_mongodb::MongodbConnectionManager;
use typemap::Key as TypeMapKey;

pub struct MongoContainer;

impl TypeMapKey for MongoContainer {
    type Value = Pool<MongodbConnectionManager>;
}
