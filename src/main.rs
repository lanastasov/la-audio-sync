use actix_files::Files;
use actix_web::{web, App, HttpServer, Responder};
use serde::Serialize;
use std::io::{self, Write};
use tokio::net::UnixStream;
use tokio::io::{AsyncWriteExt,AsyncBufReadExt, BufReader};
use serde_json::json;

static MPV_SOCKET_PATH: &str = "/tmp/mpv-socket";

#[derive(Serialize)]
struct PlaybackTimeResponse {
    playback_time: f64,
}

// Function to communicate with mpv via Unix socket and get playback time
async fn get_playback_time() -> io::Result<f64> {
    let mut stream = UnixStream::connect(MPV_SOCKET_PATH).await?;
    let command = json!({
        "command": ["get_property", "playback-time"]
    })
    .to_string()
    + "\n";

    // Send the command to mpv
    stream.write_all(command.as_bytes()).await?;

    // Read the response
    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader.read_line(&mut response).await?;

    // Parse the JSON response to extract playback time
    let json_response: serde_json::Value = serde_json::from_str(&response)?;
    if let Some(playback_time) = json_response["data"].as_f64() {
        Ok(playback_time)
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "Failed to parse playback time"))
    }
}

// HTTP handler to return playback time
async fn playback_time_handler() -> impl Responder {
    match get_playback_time().await {
        Ok(playback_time) => web::Json(PlaybackTimeResponse { playback_time }),
        Err(e) => {
            eprintln!("Error fetching playback time: {:?}", e);
            web::Json(PlaybackTimeResponse { playback_time: -1.0 })
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/current-time", web::get().to(playback_time_handler))
            .service(Files::new("/", "./").index_file("index.html"))
            // Serve the index.html and other files from the current directory
            // Endpoint to get the playback time from mpv
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}
