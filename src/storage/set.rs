use crate::storage::schema::env::dsl as env;
use diesel::dsl::update;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::{QueryDsl, RunQueryDsl, SqliteConnection};

pub fn set_value(
    key: &str,
    value: &str,
    conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
) -> QueryResult<usize> {
    update(env::env)
        .set(env::value.eq(value))
        .filter(env::key.eq(key))
        .execute(conn)
}
