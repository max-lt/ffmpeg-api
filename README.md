[github-license-url]: /LICENSE
[action-docker-url]: https://github.com/max-lt/ffmpeg-api/actions/workflows/docker.yml
[github-container-url]: https://github.com/max-lt/ffmpeg-api/pkgs/container/ffmpeg-api

# FFmpeg API

[![License](https://img.shields.io/github/license/maxx-t/ffmpeg-api.svg)][github-license-url]
[![Build Status](https://github.com/max-lt/ffmpeg-api/actions/workflows/docker.yml/badge.svg)][action-docker-url]
[![Build Status](https://ghcr-badge.deta.dev/max-lt/ffmpeg-api/size)][action-docker-url]

This is a simple Deno server that uses FFmpeg to convert audio files from OGG format to MP3 format. The server listens on port `8080` and provides an endpoint to upload OGG files and receive the converted MP3 files as a response.

## Requirements

- [Deno](https://deno.land/) runtime
- [FFmpeg](https://ffmpeg.org/) installed on your system

## Using the Docker image

You can use the Github Container Registry to pull the [Docker image](https://github.com/max-lt/ffmpeg-api/pkgs/container/ffmpeg-api):

```
docker pull ghcr.io/max-lt/ffmpeg-api:latest
```
  
Then, run the container:

```
docker run -p 8080:8080 ghcr.io/max-lt/ffmpeg-api:latest
```

## Installation

1. Ensure you have Deno installed on your system. You can check the [Deno installation guide](https://deno.land/manual/getting_started/installation) for detailed instructions.

2. Clone this repository:

   ```
   git clone https://github.com/max-lt/ffmpeg-api.git
   ```

3. Change to the repository directory:

   ```
   cd ffmpeg-api
   ```

## Usage

1. Start the server by running:

   ```
   deno run --allow-net --allow-run=/usr/bin/ffmpeg src/main.ts
   ```

   The server will start and listen on port `8080`. You should see the following output:

   ```
   HTTP server running. Access it at: http://localhost:8080/
   ```

2. To convert an OGG file, send a `POST` request to the `/ogg-to-mp3` endpoint with the OGG file as the request body. The response will contain the converted MP3 file.

   For example, you can use `curl` to test the server:

   ```
   curl -X POST -H "Content-Type: audio/ogg" --data-binary "@sample.oga" http://localhost:8080/ogg-to-mp3 --output sample.mp3
   ```

   Replace `sample.oga` with the path to your OGG file, and `sample.mp3` with the desired output file name.

## License

This project is licensed under the [MIT License](LICENSE).
