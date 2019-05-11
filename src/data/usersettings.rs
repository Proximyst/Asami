#![allow(dead_code)]

use crate::prelude::*;
use getset::Getters;
use lru_time_cache::LruCache;
use parking_lot::RwLock;
use serde::Serialize;
use std::sync::Arc;
use typemap::Key as TypeMapKey;

pub struct UserSettingsContainer;

impl TypeMapKey for UserSettingsContainer {
    type Value = LruCache<u64, Arc<RwLock<UserSettings>>>;
}

#[derive(Getters, Serialize)]
#[get = "pub"]
pub struct UserSettings {
    user_id: u64,
    blacklisted: bool,

    #[serde(skip)]
    modified: bool,
    #[serde(skip)]
    serenity_data: Arc<RwLock<typemap::ShareMap>>,
}

impl UserSettings {
    pub fn new(
        user_id: u64,
        sharemap: &Arc<RwLock<typemap::ShareMap>>,
    ) -> Result<Arc<RwLock<Self>>> {
        let sharemap = Arc::clone(sharemap);
        let mut read = sharemap.write();
        {
            let cache = read.get_mut::<UserSettingsContainer>().failure()?;
            if let Some(s) = cache.get(&user_id) {
                return Ok(Arc::clone(s));
            }
        }

        {
            use crate::scheme::user_settings::dsl::*;
            use diesel::prelude::*;

            let pgpool = read.get::<PostgreSqlContainer>().failure()?;
            let pgconn = pgpool.get()?;

            let mut setting = user_settings
                .filter(id.eq(user_id as i64))
                .limit(1)
                .load::<(i64, bool)>(&pgconn)?;
            let setting = setting.pop();

            if let Some(s) = setting {
                let (user_id, blacklist) = s;
                let user_id = user_id as u64;

                let settings = Arc::new(RwLock::new(UserSettings {
                    user_id,
                    blacklisted: blacklist,

                    modified: false,
                    serenity_data: Arc::clone(&sharemap),
                }));

                let cache = read.get_mut::<UserSettingsContainer>().failure()?;
                cache.insert(user_id, Arc::clone(&settings));

                return Ok(settings);
            }
        }

        drop(read);

        let mut settings = UserSettings {
            user_id,
            blacklisted: false,

            modified: true,
            serenity_data: Arc::clone(&sharemap),
        };
        settings.save()?;
        let arced = Arc::new(RwLock::new(settings));

        let mut read = sharemap.write();
        let cache = read.get_mut::<UserSettingsContainer>().failure()?;

        cache.insert(user_id, Arc::clone(&arced));

        Ok(arced)
    }

    pub fn set_blacklisted(&mut self, new: bool) {
        self.modified = true;
        self.blacklisted = new;
    }

    pub fn save(&mut self) -> Result<()> {
        use crate::scheme::user_settings::dsl::*;
        use diesel::{dsl::*, prelude::*};
        if !self.modified {
            return Ok(());
        }

        let read = self.serenity_data.read();
        let connpool = read.get::<PostgreSqlContainer>().failure()?;
        let pgconn = connpool.get()?;
        insert_into(user_settings)
            .values((id.eq(self.user_id as i64), blacklisted.eq(self.blacklisted)))
            .on_conflict(id)
            .do_update()
            .set(blacklisted.eq(self.blacklisted))
            .execute(&pgconn)?;

        self.modified = false;
        Ok(())
    }

    pub fn delete(mut self) -> Result<()> {
        use crate::scheme::user_settings::dsl::*;
        use diesel::{dsl::*, prelude::*};

        self.modified = false;

        let mut read = self.serenity_data.write();

        {
            let connpool = read.get::<PostgreSqlContainer>().failure()?;
            let pgconn = connpool.get()?;
            delete(user_settings.filter(id.eq(self.user_id as i64))).execute(&pgconn)?;
        }

        {
            let cache = read.get_mut::<UserSettingsContainer>().failure()?;
            cache.remove(&self.user_id);
        }

        Ok(())
    }
}

impl Drop for UserSettings {
    fn drop(&mut self) {
        if self.modified {
            match self.save() {
                Ok(()) => {}
                Err(e) => {
                    error!(
                        "An error occurred while saving {}'s profile upon Drop: {:?}",
                        self.user_id, e
                    );
                }
            }
        }
    }
}
