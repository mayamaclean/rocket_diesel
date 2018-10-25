#![allow(proc_macro_derive_resolution_fallback)]

use super::*;
use super::schema::*;

#[derive(Queryable)]
#[derive(Serialize)]
pub struct Entry {
    pub id: i32,
    pub opt: Option<String>,
    pub num: std::time::SystemTime,
    pub hash: String,
}

#[derive(Insertable)]
#[table_name="entries"]
pub struct NewEntry<'a> {
    pub opt: &'a str,
    pub num: std::time::SystemTime,
    pub hash: &'a str,
}
