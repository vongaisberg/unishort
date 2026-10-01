use rocket::Route;
use rocket::State;
use rocket_dyn_templates::{context, Template};

use crate::ServerUrl;

#[get("/about")]
fn about(server_url: &State<ServerUrl>) -> Template {
    Template::render("blog/about", context! {server_url: &server_url.0})
}
#[get("/top-10-shortest-url-shorteners-2023")]
fn list(server_url: &State<ServerUrl>) -> Template {
    Template::render("blog/list", context! {server_url: &server_url.0})
}

pub fn get_routes() -> Vec<Route> {
    routes![about, list]
}
