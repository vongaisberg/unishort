#![feature(proc_macro_hygiene, decl_macro, async_fn_in_trait)]

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate unicode_names2;
extern crate url;
#[macro_use]
extern crate lazy_static;

pub mod codepoint_list;
pub mod db;
pub mod models;
pub mod schema;
pub mod unicode_table;
pub mod url_codepoint;

use diesel::prelude::*;
use rocket::form::Form;
use rocket::fs::NamedFile;
use rocket::response::Redirect;
use rocket::State;

use diesel::dsl::count_star;
use diesel::insert_into;
use models::Url;
use rocket::http::Status;
use rocket::response::status;
use rocket_dyn_templates::context;
use rocket_dyn_templates::Template;
use schema::urls::dsl::*;

use dotenv::dotenv;
use regex::Regex;
use std::env;

static URL_REGEX: &str = "^([Hh][Tt][Tt][Pp][Ss]?:\\/\\/)?(?:(?:[a-zA-Z\\u00a1-\\uffff0-9]+-?)*[a-zA-Z\\u00a1-\\uffff0-9-]+)(?:\\.(?:[a-zA-Z\\u00a1-\\uffff0-9]+-?)*[a-zA-Z\\u00a1-\\uffff0-9]+)*(?:\\.(?:[a-zA-Z\\u00a1-\\uffff]{2,}))(?::\\d{2,5})?(?:\\/[^\\s]*)?$";
lazy_static! {
    static ref HTTP_REGEX: Regex = Regex::new(r"^https?:\/\/").unwrap();
}

#[derive(FromForm)]
struct ShortenTask {
    url_long: String,
}

#[get("/")]
fn index(db: db::Connection) -> Template {
    let links = urls
        .order_by(timestamp.desc())
        .limit(10)
        .load::<Url>(db.connection())
        .unwrap();
    let count = urls
        .select(count_star())
        .first::<i64>(db.connection())
        .unwrap();
    println!("{}", env::var("URL").unwrap());
    Template::render(
        "index_new",
        context! {regex: URL_REGEX, url_counter: count, links: links, server_url: env::var("URL").unwrap()},
    )
}
#[get("/styles.css")]
async fn css() -> Result<NamedFile, std::io::Error> {
    NamedFile::open("static/styles2.css").await
}

#[post("/", data = "<url_long>")]
fn shorten(
    url_long: Form<ShortenTask>,
    db: db::Connection,
    generator: &State<url_codepoint::CodepointGenerator>,
) -> Result<Template, status::Custom<String>> {
    let mut url_long = url_long.url_long.to_lowercase().clone();
    println!("{}", url_long);
    if !HTTP_REGEX.is_match(&url_long) {
        url_long = "https://".to_owned() + &url_long;
    }
    return match url::Url::parse(&url_long) {
        Err(_) => Err(status::Custom(
            Status::UnprocessableEntity,
            "The URL you entered was not valid.".to_owned(),
        )),
        Ok(_) => {
            let existing_short_url = urls
                .filter(url.eq(&url_long.to_string()))
                .first::<Url>(db.connection());
            match existing_short_url {
                Ok(_existing_short_url) => Ok(index(db)),
                Err(_) => {
                    //No short url exists for this url, generate a new one.
                    let chars = [generator.random_codepoint(), generator.random_codepoint()];
                    let url_short: String = chars[0].to_string() + &chars[1].to_string();

                    insert_into(urls)
                        .values((
                            url.eq(&url_long.to_string()),
                            short_url.eq(url_short.clone()),
                            timestamp.eq(chrono::offset::Utc::now().naive_utc()),
                        ))
                        .execute(db.connection())
                        .expect("Could not insert into DB");

                    Ok(index(db))
                }
            }
        }
    };
}

#[get("/<url_short>")]
fn resolve(
    url_short: String,
    db: db::Connection,
    _generator: &State<url_codepoint::CodepointGenerator>,
) -> Option<Redirect> {
    let _ = diesel::update(urls)
        .filter(short_url.eq(&url_short))
        .set(clicks.eq(clicks + 1))
        .execute(db.connection());
    urls.filter(short_url.eq(&url_short))
        .first::<Url>(db.connection())
        .ok()
        .map(|result| Redirect::to(result.url))
}

#[launch]
fn rocket() -> _ {
    dotenv().ok();
    println!("Display URL is set to {}", env::var("URL").unwrap());

    rocket::build()
        .manage(db::initialize(15))
        .manage(url_codepoint::CodepointGenerator::new())
        .attach(Template::fairing())
        .mount("/", routes![resolve, index, shorten, css])
}
