use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::log::private::debug;
use rocket::serde::json::Json;
use rocket::{Request, Response};

// #[macro_use]
// extern crate rocket;


/// Catches all OPTION requests in order to get the CORS related Fairing triggered.
#[options("/<_..>")]
pub fn all_options() {
    /* Intentionally left empty */
}

pub struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Cross-Origin-Resource-Sharing Fairing",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, PATCH, PUT, DELETE, HEAD, OPTIONS, GET",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[get("/history", format = "application/json")]
pub fn get_all2(group: String, ) -> Json<rocket::serde::json::Value> {
    let users = Energy::get_all();
    Json(json!({
        "status": 200,
        "result": users,
    }))
}

#[get("/current", format = "application/json")]
pub fn current() -> Json<rocket::serde::json::Value> {
    let users = Energy::get_now();
    Json(json!({
        "status": 200,
        "result": users,
    }))
}
