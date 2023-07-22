use rocket::form::Form;
use rocket::fs::NamedFile;
use rocket::response::Redirect;
use rocket::Route;
use rocket::State;

use rocket_dyn_templates::Template;

// #[get("/")]
// fn index(db: db::Connection) -> Template {
//     render_template(db, false)
// }

#[get("/about")]
fn about() -> Template {
    Template::render("blog/template", {})
}

#[get("/styles.css")]
async fn css() -> Result<NamedFile, std::io::Error> {
    NamedFile::open("static/styles2.css").await
}

pub fn get_routes() -> Vec<Route> {
    routes![about]
}
