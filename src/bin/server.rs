#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate diesel;
extern crate diesel_rocket;
extern crate dotenv;
extern crate rocket;

use dotenv::dotenv;
use diesel::{PgConnection};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};
use std::env;
use std::ops::Deref;
use std::time;

fn init_pool() -> diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<PgConnection>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    diesel_rocket::PgPool::from(&database_url).pool
}

pub struct DbConn(pub PooledConnection<ConnectionManager<PgConnection>>);

impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let pool = request.guard::<State<diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<PgConnection>>>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(DbConn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ()))
        }
    }
}

impl Deref for DbConn {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[get("/")]
fn root(conn: DbConn) -> String {
    let timer = time::Instant::now();
    let resp  = diesel_rocket::get_entries(
                    &*conn,
                    time::SystemTime::UNIX_EPOCH,
                    time::SystemTime::now());

    println!("{:?}", timer.elapsed());

    resp
}

#[get("/<i>")]
fn index(i: usize, conn: DbConn) -> String {
    let timer = time::Instant::now();
    let cnt = diesel_rocket::get_count(&*conn);
    let n: i32;
    if i < cnt { n = (i % cnt) as i32; }
    else { n = i as i32; }

    let r = diesel_rocket::get_entry(&*conn, n);
    println!("{:?}", timer.elapsed());

    r
}

#[get("/rm/<i>")]
fn remove(i: usize, conn: DbConn) -> String {
    let timer = time::Instant::now();
    let cnt = diesel_rocket::get_count(&*conn);
    let n: i32;
    if i < cnt { n = cnt as i32; }
    else { n = i as i32; }

    let r = diesel_rocket::rm_entry(&*conn, n);
    println!("{:?}", timer.elapsed());

    if r == 0 { return format!("no record {} (reported deletion of {} records)", n, r); }
    format!("deleted {}", n)
}

fn main() {
    rocket::ignite()
        .manage(init_pool())
        .mount("/", routes![index, root, remove])
        .launch();
}
