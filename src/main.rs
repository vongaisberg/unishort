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

// static URL_REGEX: &str = "^([Hh][Tt][Tt][Pp][Ss]?:\\/\\/)?(?:(?:[a-zA-Z\\u00a1-\\uffff0-9]+-?)*[a-zA-Z\\u00a1-\\uffff0-9]+)(?:\\.(?:[a-zA-Z\\u00a1-\\uffff0-9]+-?)*[a-zA-Z\\u00a1-\\uffff0-9]+)*(?:\\.(?:[a-zA-Z\\u00a1-\\uffff]{2,}))(?::\\d{2,5})?(?:\\/[^\\s]*)?$";
// static URL_REGEX: &str = "^((https?):\\/\\/)?([\\w_-\\u00a1-\\uffff]+(?:(?:\\.[\\w_-\\u00a1-\\uffff]+)+))([\\u00a1-\\uffff\\w.,@?^=%&:\\/~+#-]*[\\u00a1-\\uffff\\w@?^=%&\\/~+#-])$";

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
    // content::RawHtml(format!(
    //     "{}{}",
    //     URL_REGEX,
    //     urls.select(count_star())
    //         .first::<i64>(db.connection())
    //         .unwrap()
    // ));

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
    // Ok(content::RawCss(
    //     include_str!("../templates/index.css").to_owned(),
    // ))

    NamedFile::open("static/styles2.css").await
}

#[post("/", data = "<url_long>")]
fn shorten(
    url_long: Form<ShortenTask>,
    db: db::Connection,
    generator: &State<url_codepoint::CodepointGenerator>,
    //  url_verifier: State<Regex>,
) -> Result<Template, status::Custom<String>> {
    let mut url_long = url_long.url_long.to_lowercase().clone();
    println!("{}", url_long);
    if !HTTP_REGEX.is_match(&url_long) {
        println!("No http");
        url_long = "https://".to_owned() + &url_long;
    }
    println!("{}", url_long);
    return match url::Url::parse(&url_long) {
        //let ok: Result<String, String> = Ok(url_long.to_owned());
        //return match ok{
        Err(_) => Err(status::Custom(
            Status::UnprocessableEntity,
            "The URL you entered was not valid.".to_owned(),
        )),
        Ok(_) => {
            let existing_short_url = urls
                .filter(url.eq(&url_long.to_string()))
                .first::<Url>(db.connection());
            match existing_short_url {
                Ok(existing_short_url) => {
                    // let name1 = unicode_names2::name(
                    //     existing_short_url.short_url.chars().nth(0).unwrap_or('\0'),
                    // )
                    // .map(|name| name.to_string())
                    // .unwrap_or("<invalid>".to_owned())
                    // .to_lowercase();
                    // let name2 = unicode_names2::name(
                    //     existing_short_url.short_url.chars().nth(1).unwrap_or('\0'),
                    // )
                    // .map(|name| name.to_string())
                    // .unwrap_or("<invalid>".to_owned())
                    // .to_lowercase();
                    // return Ok(content::RawHtml(format!(
                    //     include_str!("../templates/result.html"),
                    //     server_url, existing_short_url.short_url, name1, name2
                    // )));
                    Ok(index(db))
                }
                Err(_) => {
                    //No short url exists for this url, generate a new one.
                    let character1 = generator.random_codepoint();
                    let character2 = generator.random_codepoint();
                    let url_short: String = character1.to_string() + &character2.to_string();
                    println!("Character1: U+{:X}", character1 as u32);
                    println!("Character2: U+{:X}", character2 as u32);

                    println!(
                        "Inserted, result was {:?}",
                        insert_into(urls)
                            .values((
                                url.eq(&url_long.to_string()),
                                short_url.eq(url_short.clone()),
                                timestamp.eq(chrono::offset::Utc::now().naive_utc()),
                            ))
                            .execute(db.connection())
                    );

                    // let name1 = unicode_names2::name(character1)
                    //     .map(|name| name.to_string())
                    //     .unwrap_or("<invalid>".to_owned())
                    //     .to_lowercase();
                    // let name2 = unicode_names2::name(character2)
                    //     .map(|name| name.to_string())
                    //     .unwrap_or("<invalid>".to_owned())
                    //     .to_lowercase();

                    // println!(
                    //     "Character: {} ({}) [U+{:X}] ({}) [U+{:X}]",
                    //     url_short, name1, character1 as u32, name2, character2 as u32
                    // );
                    // return Ok(content::RawHtml(format!(
                    //     include_str!("../templates/result.html"),
                    //     server_url, url_short, name1, name2
                    // )));
                    Ok(index(db))
                }
            }
        }
    };

    //let character = url_codepoint::random_codepoint();
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
        // .mount("/static", StaticFiles::from("/static"))
        //.manage(Regex::new(r"^(?:http(s)?://)?[\w.-]+(?:\.[\w\.-]+)+[\w\-\._~:/?#[\]@!\$&'\(\)\*\+,;=.]+$").unwrap())
        //.mount("/shorten", routes![shorten])
        .mount("/", routes![resolve, index, shorten, css])
}
