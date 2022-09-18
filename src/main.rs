mod argo;
mod git;
mod python;
mod runner;

#[macro_use]
extern crate rocket;

use argo::{error_response, ExecuteTemplateArgs, ExecuteTemplateReply};

use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    serde::json::Json,
    Request,
};
use runner::extract_runtime;
use std::{env, fs};

#[derive(Debug)]
struct ApiKey<'r>(&'r str);

#[derive(Debug)]
enum ApiKeyError {
    Missing,
    Unauthorized,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey<'r> {
    type Error = ApiKeyError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        /// Returns true if `key` is a valid API key string.
        fn is_valid<'r>(req: &'r Request<'_>, key: &str) -> bool {
            return key.split(' ').last() == Some(req.rocket().state::<String>().unwrap());
        }

        match req.headers().get_one("Authorization") {
            None => Outcome::Failure((Status::BadRequest, ApiKeyError::Missing)),
            Some(key) if is_valid(req, key) => Outcome::Success(ApiKey(key)),
            Some(_) => Outcome::Failure((Status::Unauthorized, ApiKeyError::Unauthorized)),
        }
    }
}

#[post(
    "/api/v1/template.execute",
    format = "application/json",
    data = "<input>"
)]
fn index(_key: ApiKey, input: Json<ExecuteTemplateArgs>) -> Json<ExecuteTemplateReply> {
    info!("Got a request {:?}", input);
    let runtime = extract_runtime(&input);
    if let Err(e) = runtime {
        return Json(error_response(&format!(
            "No runtime found for: {}, error: {}",
            input.0.template.plugin.containerless.runtime, e
        )));
    }
    let result = match runtime.unwrap().handle_request(&input) {
        Ok(result) => result,
        Err(error) => error_response(&format!("Cannot run script: {}", error)),
    };
    info!("Sending response {:?}", result);
    Json(result)
}

#[launch]
fn rocket() -> _ {
    env_logger::init();
    let token = fs::read_to_string(
        env::var("ARGO_TOKEN_PATH").unwrap_or("/var/run/argo/token".to_string()),
    )
    .expect("cannot read token");
    rocket::build().mount("/", routes![index]).manage(token)
}
