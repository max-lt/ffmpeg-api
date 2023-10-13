use actix_web::{error, http::header, post, web, App, Error, HttpServer, Responder};
use futures::StreamExt;
use std::io::Read;
use std::io::Write;
use std::process::{Command, Stdio};
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
    let mut ffmpeg = Command::new("/usr/bin/ffmpeg")
        .args(&[
            "-i",
            "pipe:0",
            "-codec:a",
            "libmp3lame",
            "-loglevel",
            "debug",
            "-f",
            match format {
                AudioFormat::Wav => "wav",
                AudioFormat::Mp3 => "mp3",
            },
            "pipe:1",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| {
            eprintln!("Error running FFmpeg: {:?}", e);
            error::ErrorInternalServerError("Internal Server Error")
        })?;

    let stdin = ffmpeg
        .stdin
        .take()
        .ok_or_else(|| error::ErrorInternalServerError("Failed to open stdin"))?;

    let stdout = ffmpeg
        .stdout
        .take()
        .ok_or_else(|| error::ErrorInternalServerError("Failed to open stdout"))?;

    let ogg_data = ogg_data.to_vec();
    let stdin = actix_web::rt::spawn(async move {
        let mut stdin = std::io::BufWriter::new(stdin);
        stdin.write_all(&ogg_data)?;
        stdin.flush()?;
        Result::<(), Error>::Ok(())
    });

    let stdout = actix_web::rt::spawn(async move {
        let mut stdout = std::io::BufReader::new(stdout);
        let mut wav_data = Vec::new();
        stdout.read_to_end(&mut wav_data)?;
        Result::<Vec<u8>, Error>::Ok(wav_data)
    });

    let (_, stdout_result) = futures::join!(stdin, stdout);

    // Ensure FFmpeg exited successfully
    let ffmpeg_result = ffmpeg.wait().map_err(|e| {
        eprintln!("Error waiting for FFmpeg: {:?}", e);
        error::ErrorInternalServerError("Internal Server Error")
    })?;

    if !ffmpeg_result.success() {
        eprintln!("FFmpeg exited with error code: {:?}", ffmpeg_result.code());
        return Err(error::ErrorInternalServerError("Internal Server Error"));
    }

    let wav_data = stdout_result.map_err(|e| {
        eprintln!("Join error for stdout: {:?}", e);
        error::ErrorInternalServerError("Internal Server Error")
    })?;

    match wav_data {
        Ok(wav_data) => Ok(Bytes::from(wav_data)),
        Err(e) => {
            eprintln!("Error reading stdout: {:?}", e);
            Err(error::ErrorInternalServerError("Internal Server Error"))
        }
    }
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
    HttpServer::new(|| App::new().service(ogg_to_wav).service(ogg_to_mp3))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
