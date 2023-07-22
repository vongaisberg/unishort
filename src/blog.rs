use rocket::fs::NamedFile;
use rocket::Route;

use rocket_dyn_templates::Template;

// #[get("/")]
// fn index(db: db::Connection) -> Template {
//     render_template(db, false)
// }

#[get("/about")]
fn about() -> Template {
    Template::render("blog/about", ())
}
#[get("/top-10-shortest-url-shorteners-2023")]
fn list() -> Template {
    Template::render("blog/list", ())
}


pub fn get_routes() -> Vec<Route> {
    routes![about, list]
}
