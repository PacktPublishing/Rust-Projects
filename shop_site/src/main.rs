use actix::prelude::*;
use actix_web::middleware::session::{CookieSessionBackend, RequestSession, SessionStorage};
use actix_web::{
    fs, http, server::HttpServer, App, AsyncResponder, Form, HttpRequest, HttpResponse, Query,
    Responder, State,
};
use failure::Error;
use futures::Future;
use handlebars::Handlebars;
use serde_derive::*;
use shop_base::{Conn, Item};

enum DBRequest {
    FindItems(String, i64),
}
enum DBResponse {
    FoundItems(Vec<Item>),
}

impl Message for DBRequest {
    type Result = Result<DBResponse, Error>;
}

pub struct DBExecutor {
    conn: Conn,
}

impl Actor for DBExecutor {
    type Context = SyncContext<Self>;
}

impl Handler<DBRequest> for DBExecutor {
    type Result = Result<DBResponse, Error>;
    fn handle(&mut self, msg: DBRequest, _: &mut Self::Context) -> Self::Result {
        match msg {
            DBRequest::FindItems(s, i) => Ok(DBResponse::FoundItems(self.conn.find_items(&s, i)?)),
        }
    }
}

#[derive(Deserialize, Debug)]
struct FormFindItems {
    search_term: String,
    limit: Option<i64>,
}

fn search<F>(
    page_hand: &Addr<DBExecutor>,
    ffi: &FormFindItems,
    req: &HttpRequest<F>,
) -> impl Responder {
    let searches = req
        .session()
        .get::<i32>("searches")
        .expect("Session Should exist")
        .unwrap_or(0)
        + 1;
    req.session()
        .set("searches", searches)
        .expect("Could not set searches");
    page_hand
        .send(DBRequest::FindItems(
            ffi.search_term.clone(),
            ffi.limit.unwrap_or(5),
        ))
        .and_then(move |r| match r {
            Ok(DBResponse::FoundItems(v)) => Ok(HttpResponse::Ok()
                .content_type("text/html")
                .body(TEMPLATES.render("item_list", &(&v, searches)).unwrap())),
            Err(_) => Ok(HttpResponse::Ok().json("Error finding Database")),
        })
        .responder()
}

lazy_static::lazy_static! {
    static ref TEMPLATES:Handlebars = {
        let mut res = Handlebars::new();
        let df = std::fs::read_to_string("test_site/templates/item_list.html").expect("Could not read template");
        res.register_template_string("item_list",df).expect("Could not parse template");
        res
    };
}

fn main() {
    let sys = System::new("shop_site");

    let db_hand = SyncArbiter::start(3, || DBExecutor {
        conn: Conn::new().unwrap(),
    });

    HttpServer::new(move || {
        vec![
            App::with_state(db_hand.clone())
                .prefix("/db/")
                .middleware(SessionStorage::new(
                    CookieSessionBackend::signed(&[0; 32]).secure(false),
                ))
                .resource("/", |r| {
                    r.f(|_| {
                        HttpResponse::Ok()
                            .content_type("text/plain")
                            .body("This is the Database side of the app")
                    })
                })
                .resource("/find_items", |r| {
                    r.method(http::Method::GET).with(
                        |(state, query, req): (
                            State<Addr<DBExecutor>>,
                            Query<FormFindItems>,
                            HttpRequest<_>,
                        )| { search(&state, &query, &req) },
                    );

                    r.method(http::Method::POST).with(
                        |(state, form, req): (
                            State<Addr<DBExecutor>>,
                            Form<FormFindItems>,
                            HttpRequest<_>,
                        )| { search(&state, &form, &req) },
                    )
                })
                .boxed(),
            App::new()
                .handler(
                    "/",
                    fs::StaticFiles::new("test_site/static/")
                        .unwrap()
                        .show_files_listing()
                        .index_file("index.html"),
                )
                .boxed(),
        ]
    })
    .bind("127.0.0.1:8088")
    .unwrap()
    .start();

    sys.run();

    println!("Done");
}
