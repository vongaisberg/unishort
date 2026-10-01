#[macro_use]
extern crate rocket;

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate lazy_static;

pub mod blog;
pub mod db;
pub mod favicon;
pub mod models;
pub mod qr;
pub mod schema;
pub mod target_url;
pub mod url_codepoint;

use diesel::insert_into;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::fs::FileServer;
use rocket::http::{ContentType, Header, Status};
use rocket::request::FlashMessage;
use rocket::response::{status, Flash, Redirect};
use rocket::State;
use rocket_dyn_templates::{context, Template};
use serde_derive::Serialize;

use db::PgPool;
use models::Url;
use schema::urls::dsl::*;

use dotenv::dotenv;
use std::env;

/// How often `shorten` draws a new short code after hitting one that is
/// already taken before giving up.
const MAX_SHORTEN_ATTEMPTS: usize = 10;

/// Everything a page loads comes from this origin. Besides keeping visitors'
/// browsers from talking to third parties, this stops injected markup from
/// running script, since there is no inline script to allow.
const CONTENT_SECURITY_POLICY: &str = "default-src 'none'; script-src 'self'; \
    style-src 'self'; img-src 'self'; form-action 'self'; base-uri 'none'; \
    frame-ancestors 'none'";

embed_migrations!();

/// The public base URL short links are displayed with, from the `URL` env var.
pub struct ServerUrl(pub String);

#[derive(FromForm)]
struct ShortenTask {
    url_long: String,
}

/// A row of the recent links table.
#[derive(Serialize)]
struct LinkView {
    #[serde(flatten)]
    link: Url,
    /// The target with its domain in Unicode instead of punycode.
    display_url: String,
    /// The target's host, for /favicon/<host>.
    favicon_host: Option<String>,
}

#[get("/")]
async fn index(
    pool: &State<PgPool>,
    server_url: &State<ServerUrl>,
    flash: Option<FlashMessage<'_>>,
) -> Result<Template, Status> {
    let links = db::run(pool, |conn| {
        urls.order_by(timestamp.desc()).limit(10).load::<Url>(conn)
    })
    .await?
    .map_err(|e| {
        log::error!("Could not load recent links: {}", e);
        Status::InternalServerError
    })?;
    let links: Vec<LinkView> = links
        .into_iter()
        .map(|link| LinkView {
            display_url: target_url::display(&link.url),
            favicon_host: target_url::host(&link.url),
            link,
        })
        .collect();
    // Set by `shorten`: either highlight the link that was just created, or
    // say why the submitted URL was refused. Either way only once.
    let new = flash.as_ref().map_or(false, |f| f.kind() == "new");
    let error = flash
        .as_ref()
        .filter(|f| f.kind() == "error")
        .map(|f| f.message().to_owned());
    Ok(Template::render(
        "index",
        context! {links: links, server_url: &server_url.0, new: (if new {"new"} else {""}), error: error},
    ))
}

#[post("/", data = "<task>")]
async fn shorten(
    task: Form<ShortenTask>,
    pool: &State<PgPool>,
    generator: &State<url_codepoint::CodepointGenerator>,
) -> Result<Flash<Redirect>, status::Custom<&'static str>> {
    let url_long = match target_url::normalize(&task.url_long) {
        Some(target) => String::from(target),
        None => {
            return Ok(Flash::error(
                Redirect::to(uri!(index)),
                "That doesn't look like a web address. Try something like example.com/page.",
            ))
        }
    };
    let internal_error = status::Custom(
        Status::InternalServerError,
        "Something went wrong, please try again.",
    );

    // Short codes are random, so a new one can collide with an existing one.
    // The UNIQUE constraint on short_url catches that; draw again.
    for _ in 0..MAX_SHORTEN_ATTEMPTS {
        let url_short: String = [generator.random_codepoint(), generator.random_codepoint()]
            .iter()
            .collect();
        let row = (
            url.eq(url_long.clone()),
            short_url.eq(url_short),
            timestamp.eq(chrono::offset::Utc::now().naive_utc()),
        );
        let inserted = db::run(pool, move |conn| insert_into(urls).values(row).execute(conn))
            .await
            .map_err(|_| internal_error.clone())?;
        match inserted {
            Ok(_) => return Ok(Flash::new(Redirect::to(uri!(index)), "new", "")),
            Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => continue,
            Err(e) => {
                log::error!("Could not insert short link: {}", e);
                return Err(internal_error);
            }
        }
    }
    log::error!("No free short code after {} attempts", MAX_SHORTEN_ATTEMPTS);
    Err(internal_error)
}

#[get("/<url_short>")]
async fn resolve(url_short: String, pool: &State<PgPool>) -> Result<Option<Redirect>, Status> {
    db::run(pool, move |conn| {
        diesel::update(urls.filter(short_url.eq(url_short)))
            .set(clicks.eq(clicks + 1))
            .returning(url)
            .get_result::<String>(conn)
            .optional()
    })
    .await?
    // Rows from before URLs were normalized can hold raw Unicode, which is
    // not a valid Location header; re-serializing through `url` fixes that.
    .map(|target| {
        target.map(|target| {
            Redirect::to(::url::Url::parse(&target).map_or(target, String::from))
        })
    })
    .map_err(|e| {
        log::error!("Could not resolve short link: {}", e);
        Status::InternalServerError
    })
}

/// Brings the schema up to date before the first request. A failure is
/// logged but does not stop the app: the links table predates these
/// migrations, so the site keeps working, and only what a missing
/// migration adds (such as favicons) is unavailable.
async fn run_migrations(rocket: rocket::Rocket<rocket::Build>) -> rocket::Rocket<rocket::Build> {
    let pool = match rocket.state::<PgPool>() {
        Some(pool) => pool.clone(),
        None => return rocket,
    };
    let result = rocket::tokio::task::spawn_blocking(move || {
        let conn = pool.get().map_err(|e| e.to_string())?;
        embedded_migrations::run(&*conn).map_err(|e| e.to_string())
    })
    .await;
    match result {
        Ok(Ok(())) => log::info!("Database schema is up to date"),
        Ok(Err(e)) => log::error!("Could not run database migrations: {}", e),
        Err(e) => log::error!("Migration task failed: {}", e),
    }
    rocket
}

#[launch]
fn rocket() -> _ {
    dotenv().ok();
    let server_url = env::var("URL").expect("URL must be set");
    println!("Display URL is set to {}", server_url);

    rocket::build()
        .manage(db::initialize(15))
        .manage(url_codepoint::CodepointGenerator::default())
        .manage(ServerUrl(server_url))
        .manage(favicon::Client::new())
        .attach(AdHoc::on_ignite("Database migrations", run_migrations))
        .attach(Template::fairing())
        .attach(AdHoc::on_response("Content-Security-Policy", |_, response| {
            Box::pin(async move {
                if response.content_type() == Some(ContentType::HTML) {
                    response.set_header(Header::new(
                        "Content-Security-Policy",
                        CONTENT_SECURITY_POLICY,
                    ));
                }
            })
        }))
        .mount(
            "/",
            routes![resolve, index, shorten, qr::qr, favicon::favicon],
        )
        .mount("/static", FileServer::from("static"))
        .mount("/blog", blog::get_routes())
}
