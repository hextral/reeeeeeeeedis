#[macro_use]
extern crate rocket;

pub mod cors;

use cors::CORS;
use redis::Commands;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::serde::Serialize;
use rocket::{Build, Rocket, State};
use std::env;
use std::sync::RwLock;

const DEFAULT_URL: &str = "redis://localhost:6379";

#[derive(Serialize)]
struct ReeeeeeeeedisStatus {
  running: bool,
}

#[get("/")]
fn index() -> Json<ReeeeeeeeedisStatus> {
  Json(ReeeeeeeeedisStatus { running: true })
}

#[post("/data/<key>", data = "<value>")]
fn post(key: &str, value: &str, client: &State<RwLock<redis::Connection>>) -> Status {
  let mut conn = match client.write() {
    Ok(conn) => conn,
    Err(_) => return Status::InternalServerError,
  };

  match conn.set::<_, _, ()>(key, value) {
    Ok(_) => Status::Created,
    Err(_) => Status::InternalServerError,
  }
}

#[get("/data/<key>")]
fn get(key: &str, client: &State<RwLock<redis::Connection>>) -> Result<String, Status> {
  let mut conn = match client.write() {
    Ok(conn) => conn,
    Err(_) => return Err(Status::InternalServerError),
  };

  match conn.get::<_, String>(key) {
    Ok(value) => Ok(value),
    Err(_) => Err(Status::NotFound),
  }
}

#[launch]
fn rocket() -> Rocket<Build> {
  dotenv::dotenv().ok();

  let host = match env::var("REDIS_HOST") {
    Ok(host) => host,
    Err(_) => String::from(DEFAULT_URL),
  };

  let port = match env::var("REDIS_PORT") {
    Ok(port) => port.parse().unwrap(),
    Err(_) => 6379,
  };

  let username = match env::var("REDIS_USERNAME") {
    Ok(u) => u,
    Err(_) => String::from(""),
  };

  let password = match env::var("REDIS_PASSWORD") {
    Ok(p) => format!(":{}", p),
    Err(_) => String::from(""),
  };

  let separator = match !username.is_empty() || !password.is_empty() {
    true => String::from("@"),
    false => String::from(""),
  };

  let db = match env::var("REDIS_DB") {
    Ok(db) => db.parse().unwrap(),
    Err(_) => 0,
  };

  let url = format!(
    "redis://{}{}{}{}:{}/{}",
    username, password, separator, host, port, db
  );

  let client = redis::Client::open(url.clone()).expect("Failed to connect to Redis");

  let conn = client
    .get_connection()
    .expect(format!("Failed to connect to Redis at: {}", &url).as_str());

  let conn_lock = RwLock::new(conn);

  rocket::build()
    .attach(CORS) //
    .manage(conn_lock) //
    .mount("/", routes![index, post, get]) //
}
