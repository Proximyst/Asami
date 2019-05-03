#![allow(dead_code)]

use super::MongoContainer;
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

        let doc = {
            let mongo: &MongoPool = read.get::<MongoContainer>().failure()?;
            let mongo = mongo.get()?;
            let collection: mongodb::coll::Collection =
                mongo.collection(crate::consts::COLLECTION_SERVER_SETTINGS);
            collection.find_one(
                Some(doc! {
                    "server_id": server_id,
                }),
                None,
            )?
        };

        if let Some(doc) = doc {
            let blacklisted = doc.get_bool("blacklisted")?;
            let settings = Arc::new(RwLock::new(ServerSettings {
                server_id,
                blacklisted,

                modified: false,
                serenity_data: Arc::clone(&sharemap),
            }));
            let cache = read.get_mut::<ServerSettingsContainer>().failure()?;
            cache.insert(server_id, Arc::clone(&settings));
            return Ok(settings);
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
        if !self.modified {
            return Ok(());
        }

        let read = self.serenity_data.read();
        let mongo: &MongoPool = read.get::<MongoContainer>().failure()?;
        let mongo = mongo.get()?;
        let collection: mongodb::coll::Collection =
            mongo.collection(crate::consts::COLLECTION_SERVER_SETTINGS);
        collection.insert_one(
            doc! {
                "server_id": self.server_id,
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
