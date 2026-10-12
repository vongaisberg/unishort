use rocket::response::Redirect;
use rocket::Route;
use rocket::State;
use rocket_dyn_templates::{context, Template};

use crate::ServerUrl;

#[get("/about")]
fn about(server_url: &State<ServerUrl>) -> Template {
    Template::render("blog/about", context! {server_url: &server_url.0})
}
#[get("/shortest-url-shorteners")]
fn shortest(server_url: &State<ServerUrl>) -> Template {
    Template::render("blog/shortest", context! {server_url: &server_url.0})
}

/// The 2023 version of the comparison, which search engines still know.
#[get("/top-10-shortest-url-shorteners-2023")]
fn list_2023() -> Redirect {
    Redirect::moved(uri!("/blog", shortest))
}

pub fn get_routes() -> Vec<Route> {
    routes![about, shortest, list_2023]
}
