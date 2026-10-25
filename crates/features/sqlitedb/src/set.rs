use crate::init::SqliteDB;
use crate::schema::env::dsl as env;
use defs::{Result, Set};
use diesel::RunQueryDsl;
use diesel::dsl::update;
use diesel::prelude::*;

impl Set for SqliteDB {
    fn set_value(&self, key: &str, value: &str) -> Result<usize> {
        Ok(update(env::env)
            .set(env::value.eq(value))
            .filter(env::key.eq(key))
            .execute(&mut self.get_pool_connection())?)
    }
}
