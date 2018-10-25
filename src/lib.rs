#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate farmhash;
#[macro_use]
extern crate lazy_static;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::r2d2;
use farmhash::FarmHasher;
use std::hash::{Hash, Hasher};
use std::time;

pub mod schema;
pub mod models;

pub type PooledPg = r2d2::PooledConnection<r2d2::ConnectionManager<PgConnection>>;
pub struct PgPool {
    pub pool: r2d2::Pool<r2d2::ConnectionManager<PgConnection>>,
}

impl PgPool {
    pub fn from(s: &str) -> PgPool {
        let m = r2d2::ConnectionManager::new(s);
        let p = r2d2::Pool::new(m)
            .expect("Postgres connection pool could not be created");

        PgPool {
            pool: p
        }
    }

    pub fn get_connection(&self) -> Option<PooledPg> {
        let c = self.pool.get();
        if c.is_err() { return None }
        Some(c.unwrap())
    }
}

pub fn get_count(c: &PgConnection) -> usize {
    use schema::entries::dsl::*;

    let count = entries
        .count()
        .execute(c)
        .expect("DB error");

    count
}

pub fn create_entry<'a>(c: &PgConnection, o: &'a str) -> String {
    let mut h = FarmHasher::default();
    let n     = time::SystemTime::now();
    o.hash(&mut h);
    n.hash(&mut h);

    let e = models::NewEntry {
        opt: o,
        num: n,
        hash: &h.finish().to_string(),
    };

    let result: models::Entry = diesel::insert_into(schema::entries::table)
        .values(&e)
        .get_result(c)
        .expect("DB error");

    serde_json::to_string(&result).unwrap()
}

pub fn get_entries(c: &PgConnection, start: time::SystemTime, end: time::SystemTime) -> String {
    use schema::entries::dsl::*;

    let results = entries
        .filter(num.ge(start))
        .filter(num.lt(end))
        .limit(50)
        .load::<models::Entry>(c)
        .expect("DB error");

    if results.len() == 0 { return String::from("Error!"); }

    let mut resp = String::new();
    for e in results {
        resp += &serde_json::to_string(&e).unwrap();
    }
    if resp.len() == 0 { return String::from("Error!"); }

    resp
}

pub fn rm_entries(c: &PgConnection, start: time::SystemTime, end: time::SystemTime) -> usize {
    use schema::entries::dsl::*;

    diesel::delete(entries
        .filter(num.ge(start))
        .filter(num.lt(end)))
        .execute(c)
        .expect("DB error")
}

pub fn edit_entry<'a>(c: &PgConnection, i: i32, o: &'a str) -> String {
    use schema::entries::dsl::*;

    let mut h = FarmHasher::default();
    let n     = time::SystemTime::now();
    o.hash(&mut h);
    n.hash(&mut h);

    let result = diesel::update(entries
        .filter(id.eq(i)))
        .set((opt.eq(o), num.eq(n), hash.eq(h.finish().to_string())))
        .load::<models::Entry>(c)
        .expect("DB error");

    if result.len() == 0 { return String::from("Error!"); }

    let resp = serde_json::to_string(&result[0]).unwrap_or(String::from("Error!"));

    if resp.len() == 0 { return String::from("Error!"); }

    resp
}

pub fn get_entry(c: &PgConnection, i: i32) -> String {
    use schema::entries::dsl::*;

    let result = entries
        .filter(id.eq(i))
        .load::<models::Entry>(c)
        .expect("DB error");

    if result.len() == 0 {
        return String::from("Error");
    }

    serde_json::to_string(&result[0]).unwrap_or(String::from(""))
}

pub fn rm_entry(c: &PgConnection, i: i32) -> usize {
    use schema::entries::dsl::*;

    diesel::delete(entries
        .filter(id.eq(i)))
        .execute(c)
        .expect("DB error")
}

#[cfg(test)]
mod tests {
    use dotenv::dotenv;
    use std::env;
    use super::*;

    #[test]
    fn create_test() {
        dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let DB = PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url));

        let e = create_entry(&DB, "hello world");

        assert!(e.len() > 0);
        print!("\ncreated:\n{}\n", e);
    }

    #[test]
    fn gets_test() {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        let DB = PgConnection::establish(&database_url)
            .expect(&format!("Error connecting to {}", database_url));

        let e = get_entries(&DB, time::SystemTime::UNIX_EPOCH, time::SystemTime::now());
        assert!(e != String::from("Error!"));
        print!("\nget test:\n{:?}\n", e);

        let f = get_entries(&DB, time::SystemTime::now(), time::SystemTime::now());
        assert!(f == String::from("Error!"));
    }

    #[test]
    fn get_test() {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        let DB = PgConnection::establish(&database_url)
            .expect(&format!("Error connecting to {}", database_url));

        let e = get_entry(&DB, 7);
        print!("\ngetter test:\n{}\n", e);
    }

    //#[test]
    fn rm_test() {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        let DB = PgConnection::establish(&database_url)
            .expect(&format!("Error connecting to {}", database_url));

        let r = rm_entry(&DB, 13);
        assert!(r > 0);
    }

    //#[test]
    fn rm_range_test() {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        let DB = PgConnection::establish(&database_url)
            .expect(&format!("Error connecting to {}", database_url));

        let r = rm_entries(&DB, time::SystemTime::now()-time::Duration::from_secs(600), time::SystemTime::now());
        assert!(r > 0);
        print!("\n{}\n", r);
    }

    #[test]
    fn edit_test() {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        let DB = PgConnection::establish(&database_url)
            .expect(&format!("Error connecting to {}", database_url));

        let r = edit_entry(&DB, 7, "edited!");
        assert!(r != String::from("Error!"));
        print!("\nedited:\n{:?}\n", r);
    }
}
