#![allow(dead_code)]

use super::MongoContainer;
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

        let doc = {
            let mongo: &MongoPool = read.get::<MongoContainer>().failure()?;
            let mongo = mongo.get()?;
            let collection: mongodb::coll::Collection =
                mongo.collection(crate::consts::COLLECTION_USER_SETTINGS);
            collection.find_one(
                Some(doc! {
                    "user_id": user_id,
                }),
                None,
            )?
        };

        if let Some(doc) = doc {
            let blacklisted = doc.get_bool("blacklisted")?;
            let settings = Arc::new(RwLock::new(UserSettings {
                user_id,
                blacklisted,

                modified: false,
                serenity_data: Arc::clone(&sharemap),
            }));
            let cache = read.get_mut::<UserSettingsContainer>().failure()?;
            cache.insert(user_id, Arc::clone(&settings));
            return Ok(settings);
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

    pub fn save(&mut self) -> Result<()> {
        if !self.modified {
            return Ok(());
        }

        let read = self.serenity_data.read();
        let mongo: &MongoPool = read.get::<MongoContainer>().failure()?;
        let mongo = mongo.get()?;
        let collection: mongodb::coll::Collection =
            mongo.collection(crate::consts::COLLECTION_USER_SETTINGS);
        collection.insert_one(
            doc! {
                "user_id": self.user_id,
                "blacklisted": self.blacklisted,
            },
            None,
        )?;

        self.modified = false;
        Ok(())
    }
    
    pub fn set_blacklisted(&mut self, new: bool) {
        self.modified = true;
        self.blacklisted = new;
    }

    pub fn delete(mut self) -> Result<()> {
        self.modified = false;

        let read = self.serenity_data.read();
        let mongo: &MongoPool = read.get::<MongoContainer>().failure()?;
        let mongo = mongo.get()?;
        let collection: mongodb::coll::Collection =
            mongo.collection(crate::consts::COLLECTION_USER_SETTINGS);
        collection.delete_one(
            doc! {
                "user_id": self.user_id,
            },
            None,
        )?;

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
