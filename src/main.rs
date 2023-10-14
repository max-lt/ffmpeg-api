use actix_web::{error, get, http::header, post, web, App, Error, HttpServer, Responder};
use futures::StreamExt;
use web::Bytes;

// 64 MiB
const MAX_SIZE: usize = 64 * 1024 * 1024;

fn check_request(
    content_type: Option<web::Header<header::ContentType>>,
    content_length: Option<web::Header<header::ContentLength>>,
) -> Result<(), Error> {
    let content_type = match content_type {
        Some(content_type) => content_type.into_inner(),
        None => return Err(error::ErrorBadRequest("Missing Content-Type header")),
    };

    let content_length = match content_length {
        Some(content_length) => content_length.into_inner(),
        None => return Err(error::ErrorBadRequest("Missing Content-Length header")),
    };

    if content_type.to_string() != "audio/ogg" {
        return Err(error::ErrorBadRequest("Content-Type must be audio/ogg"));
    }

    if content_length.gt(&MAX_SIZE) {
        return Err(error::ErrorBadRequest("Content-Length too large"));
    }

    Ok(())
}

async fn extract_payload(mut payload: web::Payload) -> Result<Bytes, Error> {
    let mut body = web::BytesMut::new();

    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    Ok(body.freeze())
}

enum AudioFormat {
    Wav,
    Mp3,
}

async fn convert_ogg(ogg_data: Bytes, format: AudioFormat) -> Result<Bytes, Error> {
    // Run FFmpeg to convert the Ogg to WAV using stdin and stdout pipes
    let cmd = subprocess::Exec::cmd("/usr/bin/ffmpeg")
        .args(&[
            "-i",
            "pipe:0",
            "-codec:a",
            "libmp3lame",
            "-loglevel",
            "error",
            "-f",
            match format {
                AudioFormat::Wav => "wav",
                AudioFormat::Mp3 => "mp3",
            },
            "pipe:1",
        ])
        .stdin(ogg_data.to_vec())
        .capture();

    match cmd {
        Ok(cmd) => Ok(Bytes::from(cmd.stdout)),
        Err(e) => {
            eprintln!("Error running FFmpeg: {:?}", e);
            Err(error::ErrorInternalServerError("Internal Server Error"))
        }
    }
}

#[get("/version")]
async fn version() -> impl Responder {
    env!("CARGO_PKG_VERSION")
}

#[post("/ogg-to-wav")]
async fn ogg_to_wav(
    payload: web::Payload,
    content_type: Option<web::Header<header::ContentType>>,
    content_length: Option<web::Header<header::ContentLength>>,
) -> impl Responder {
    check_request(content_type, content_length)?;

    let ogg = extract_payload(payload).await?;

    let wav = convert_ogg(ogg, AudioFormat::Wav).await?;

    let res = wav.customize().insert_header(("Content-Type", "audio/wav"));

    Result::<actix_web::CustomizeResponder<Bytes>, Error>::Ok(res)
}

#[post("/ogg-to-mp3")]
async fn ogg_to_mp3(
    payload: web::Payload,
    content_type: Option<web::Header<header::ContentType>>,
    content_length: Option<web::Header<header::ContentLength>>,
) -> impl Responder {
    check_request(content_type, content_length)?;

    let ogg = extract_payload(payload).await?;

    let mp3 = convert_ogg(ogg, AudioFormat::Mp3).await?;

    let res = mp3
        .customize()
        .insert_header(("Content-Type", "audio/mpeg"));

    Result::<actix_web::CustomizeResponder<Bytes>, Error>::Ok(res)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(version)
            .service(ogg_to_wav)
            .service(ogg_to_mp3)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
