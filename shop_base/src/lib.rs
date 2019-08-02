#[macro_use]
extern crate diesel;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use failure::Error;

mod models;
pub use models::{Item, NewItem};
mod schema;
use schema::items;

pub struct Conn(PgConnection);

impl Conn {
    pub fn new() -> Result<Self, Error> {
        dotenv::dotenv().ok();
        let db_url = std::env::var("DATABASE_URL")?;
        Ok(Conn(PgConnection::establish(&db_url)?))
    }

    pub fn put_item(&self, name: &str, description: &str, price: i32) -> Result<Item, Error> {
        let nit = NewItem {
            name,
            description,
            price,
        };
        diesel::insert_into(items::table)
            .values(&nit)
            .get_result(&self.0)
            .map_err(|x| x.into())
    }

    pub fn find_items(&self, name: &str, lim: i64) -> Result<Vec<Item>, Error> {
        let lname = format!("%{}%", name);
        items::table
            .filter(items::name.ilike(lname))
            .order(items::id.desc())
            .limit(lim)
            .load(&self.0)
            .map_err(|e| e.into())
    }

    pub fn set_stock(&self, id: i32, mut n: i32, rel: bool) -> Result<Item, Error> {
        if rel {
            let items: Vec<Item> = items::table.find(id).for_update().load(&self.0)?;
            n += items[0].instock;
        }
        diesel::update(items::table::find(items::table, id))
            .set(items::instock.eq(n))
            .get_result(&self.0)
            .map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
