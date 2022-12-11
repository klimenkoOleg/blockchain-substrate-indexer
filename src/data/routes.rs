// se super::db::Conn as DbConn;
// use rocket_contrib::json::Json;
// use super::models::{User, NewUser};
// use serde_json::Value;
// use crate::models::{Energy, UserData};
use rocket::serde::json::{Value, json};
use rocket::serde::{Serialize, Deserialize, json::Json};
// use rocket::serde::{Deserialize};


#[post("/users", format = "application/json")]
pub fn get_all() -> Json<Value> {
    let users = Energy::get_all();
    Json(json!({
        "status": 200,
        "result": users,
    }))
}