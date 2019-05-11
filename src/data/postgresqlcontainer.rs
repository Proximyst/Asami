use r2d2::Pool;
use diesel::{r2d2::ConnectionManager, pg::PgConnection};
use typemap::Key as TypeMapKey;

pub struct PostgreSqlContainer;

impl TypeMapKey for PostgreSqlContainer {
    type Value = Pool<ConnectionManager<PgConnection>>;
}
