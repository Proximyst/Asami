#![allow(dead_code)]

use crate::prelude::*;
use getset::Getters;
use lru_time_cache::LruCache;
use parking_lot::RwLock;
use serde::Serialize;
use std::sync::Arc;
use typemap::Key as TypeMapKey;

pub struct ServerSettingsContainer;

impl TypeMapKey for ServerSettingsContainer {
    type Value = LruCache<u64, Arc<RwLock<ServerSettings>>>;
}

#[derive(Getters, Serialize)]
#[get = "pub"]
pub struct ServerSettings {
    server_id: u64,
    blacklisted: bool,

    #[serde(skip)]
    modified: bool,
    #[serde(skip)]
    serenity_data: Arc<RwLock<typemap::ShareMap>>,
}

impl ServerSettings {
    pub fn new(
        server_id: u64,
        sharemap: &Arc<RwLock<typemap::ShareMap>>,
    ) -> Result<Arc<RwLock<Self>>> {
        let sharemap = Arc::clone(sharemap);
        let mut read = sharemap.write();
        {
            let cache = read.get_mut::<ServerSettingsContainer>().failure()?;
            if let Some(s) = cache.get(&server_id) {
                return Ok(Arc::clone(s));
            }
        }

        {
            use crate::scheme::server_settings::dsl::*;
            use diesel::prelude::*;

            let pgpool = read.get::<PostgreSqlContainer>().failure()?;
            let pgconn = pgpool.get()?;

            let mut setting = server_settings
                .filter(id.eq(server_id as i64))
                .limit(1)
                .load::<(i64, bool)>(&pgconn)?;
            let setting = setting.pop();

            if let Some(s) = setting {
                let (server_id, blacklist) = s;
                let server_id = server_id as u64;

                let settings = Arc::new(RwLock::new(ServerSettings {
                    server_id,
                    blacklisted: blacklist,

                    modified: false,
                    serenity_data: Arc::clone(&sharemap),
                }));

                let cache = read.get_mut::<ServerSettingsContainer>().failure()?;
                cache.insert(server_id, Arc::clone(&settings));

                return Ok(settings);
            }
        }

        drop(read);

        let mut settings = ServerSettings {
            server_id,
            blacklisted: false,

            modified: true,
            serenity_data: Arc::clone(&sharemap),
        };
        settings.save()?;
        let arced = Arc::new(RwLock::new(settings));

        let mut read = sharemap.write();
        let cache = read.get_mut::<ServerSettingsContainer>().failure()?;

        cache.insert(server_id, Arc::clone(&arced));

        Ok(arced)
    }

    pub fn save(&mut self) -> Result<()> {
        use crate::scheme::server_settings::dsl::*;
        use diesel::{dsl::*, prelude::*};

        if !self.modified {
            return Ok(());
        }

        let read = self.serenity_data.read();
        let connpool = read.get::<PostgreSqlContainer>().failure()?;
        let pgconn = connpool.get()?;
        insert_into(server_settings)
            .values((
                id.eq(self.server_id as i64),
                blacklisted.eq(self.blacklisted),
            ))
            .on_conflict(id)
            .do_update()
            .set(blacklisted.eq(self.blacklisted))
            .execute(&pgconn)?;

        self.modified = false;
        Ok(())
    }

    pub fn set_blacklisted(&mut self, new: bool) {
        self.modified = true;
        self.blacklisted = new;
    }

    pub fn delete(mut self) -> Result<()> {
        use crate::scheme::server_settings::dsl::*;
        use diesel::{dsl::*, prelude::*};

        self.modified = false;

        let mut read = self.serenity_data.write();

        {
            let connpool = read.get::<PostgreSqlContainer>().failure()?;
            let pgconn = connpool.get()?;
            delete(server_settings.filter(id.eq(self.server_id as i64))).execute(&pgconn)?;
        }

        {
            let cache = read.get_mut::<ServerSettingsContainer>().failure()?;
            cache.remove(&self.server_id);
        }

        Ok(())
    }
}

impl Drop for ServerSettings {
    fn drop(&mut self) {
        if self.modified {
            match self.save() {
                Ok(()) => {}
                Err(e) => {
                    error!(
                        "An error occurred while saving {}'s server profile upon Drop: {:?}",
                        self.server_id, e
                    );
                }
            }
        }
    }
}
