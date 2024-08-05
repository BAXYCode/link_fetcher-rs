use std::{env, fmt::Display, process::Stdio};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use axum_macros::debug_handler;
use eyre::Result as EyreResult;
use serde::Deserialize;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> EyreResult<()> {
    let path_to_fetcher = env::args().nth(1).expect("Path to python program missing");
    let path_to_screenshot = env::args().nth(2).expect("Path to screenshot is missing");
    let state = PythonProgramFiles {
        link_fetcher: path_to_fetcher,
        screenshot: path_to_screenshot,
    };
    let listen_addr = "0.0.0.0:6769";

    let app = Router::new().route("/", get(links)).with_state(state);

    let listener = TcpListener::bind(listen_addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
#[debug_handler]
async fn links(
    Query(address): Query<TokenAddress>,
    State(state): State<PythonProgramFiles>,
) -> Result<Json<String>, AppError> {
    println!("endpoint hit");
    let process = tokio::process::Command::new("python3")
        .args([state.to_string(), address.to_string(), state.screenshot])
        .stdout(Stdio::piped())
        .spawn()
        .expect("could not spawn task")
        .wait_with_output()
        .await?;
    let response = std::string::String::from_utf8(process.stdout)?
        .split_inclusive("}")
        .collect::<Vec<&str>>()[0]
        .to_owned();
    Ok(Json(response))
}
#[derive(Deserialize)]
struct TokenAddress {
    address: String,
}
impl Display for TokenAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.address.clone())
    }
}

#[derive(Debug, Clone)]
struct PythonProgramFiles {
    link_fetcher: String,
    screenshot: String,
}
impl Display for PythonProgramFiles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.link_fetcher.clone())
    }
}
#[derive(Debug)]
struct AppError(eyre::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("An issue was encountered: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<eyre::Error>,
{
    fn from(value: E) -> Self {
        Self(value.into())
    }
}
