use diesel::{pg::PgConnection, r2d2::ConnectionManager};
use r2d2::Pool;
use typemap::Key as TypeMapKey;

pub struct PostgreSqlContainer;

impl TypeMapKey for PostgreSqlContainer {
    type Value = Pool<ConnectionManager<PgConnection>>;
}
