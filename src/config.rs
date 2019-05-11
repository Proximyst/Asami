use getset::Getters;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Getters)]
#[get = "pub"]
pub struct Configuration {
    token: String,
    debug_logging: bool,
    mongo_host: String,
    mongo_port: u16,
    pgsql_url: String,
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            token: String::from("TOKEN HERE"),
            debug_logging: false,
            mongo_host: String::from("localhost"),
            mongo_port: 27017,
            pgsql_url: String::from("postgresql://superuser:root@localhost:5432/asami"),
        }
    }
}
