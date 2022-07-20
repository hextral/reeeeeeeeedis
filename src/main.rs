#[macro_use]
extern crate rocket;

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
  let url = match env::var("REDIS_URL") {
    Ok(url) => url,
    Err(_) => String::from(DEFAULT_URL),
  };

  let rocket_port: u16 = match env::var("REEEEEEEEEDIS_PORT") {
    Ok(port) => port.parse().unwrap(),
    Err(_) => 8989,
  };

  let client = redis::Client::open(url.clone()).expect("Failed to connect to Redis");

  let conn = client
    .get_connection()
    .expect(format!("Failed to connect to Redis on: {}", &url).as_str());

  let conn_lock = RwLock::new(conn);

  let config = rocket::Config {
    port: rocket_port,
    ..Default::default()
  };

  rocket::build()
    .configure(config)
    .manage(conn_lock) //
    .mount("/", routes![index, post, get]) //
}
