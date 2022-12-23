use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::log::private::debug;
use rocket::serde::json::{Json, json};
use rocket::{Request, Response};
use crate::database::get_db_history;
use crate::database::get_latest_record;
use crate::database::get_date_vect_60_minutes;
use crate::database::get_date_vect_60_seconds;
use crate::database::get_date_vect_24_hours;
use crate::database::get_db_total;

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

#[get("/total/<house_name>?<mode>", format = "application/json")]
pub fn get_total(house_name: String, mode: Option<String>) -> Json<rocket::serde::json::Value> {
    if (mode.clone().is_none()) {
        return Json(json!({
            "status": 400,
            "result": format!("/history URL - empty 'mode' param "),}));
    }

    let unwrap1 = mode.unwrap();
    let mode_to_compare = unwrap1;
    let time_param = match mode_to_compare.as_str() {
        "1" => {
            // let time_grouping_ticks = get_date_vect_24_hours();
            "hours" },
        // "2" => ("%H", "-1 days"),
        "2" => ("days"),
        // "3" => ("%H", "-1 days"),
        "3" => ("months"),
        val => {
            return Json(json!({
            "status": 400,
            "result": format!("/history URL - incorrect 'mode' value: {}", val),}));
        }
    };
    let result = get_db_total(String::from(time_param), house_name);
    // let users = Energy::get_all();
    Json(json!({
        "status": 200,
        "result": result,
    }))
}

#[get("/history/<house_name>?<mode>", format = "application/json")]
pub fn get_all2(house_name: String, mode: Option<String>) -> Json<rocket::serde::json::Value> {
    if (mode.clone().is_none()) {
            return Json(json!({
            "status": 400,
            "result": format!("/history URL - empty 'mode' param "),}));
    }
    let unwrap1 = mode.unwrap();
    let mode_to_compare = unwrap1;
    let (group_param, time_back_range, time_grouping_ticks) = match mode_to_compare.as_str() {
        "1" => {
            // let time_grouping_ticks = get_date_vect_24_hours();
            ("%H", "-1 days", get_date_vect_24_hours()) },
        // "2" => ("%H", "-1 days"),
        "2" => ("%M", "-1 hours", get_date_vect_60_minutes()),
        // "3" => ("%H", "-1 days"),
        "3" => ("%M", "-15 minutes", get_date_vect_60_seconds()),
        val => {
            return Json(json!({
            "status": 400,
            "result": format!("/history URL - incorrect 'mode' value: {}", val),}));
        }
    };
    // print!("time_grouping_ticks: {:?}", time_grouping_ticks);
    // print!("input data: {}, {}", group_param, time_back_range);
    let result = get_db_history(String::from(group_param),
                                String::from(time_back_range),
                                &time_grouping_ticks, house_name);
    // let users = Energy::get_all();
    Json(json!({
        "status": 200,
        "result": result,
    }))
}

#[get("/current/<house_name>", format = "application/json")]
pub fn current(house_name: String) -> Json<rocket::serde::json::Value> {
    let result = get_latest_record(house_name);
    // let users = Energy::get_now();
    Json(json!({
        "status": 200,
        "result": result,
    }))
}
