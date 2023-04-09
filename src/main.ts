import { copy } from 'https://deno.land/std@0.182.0/streams/copy.ts';
import { readerFromIterable } from 'https://deno.land/std@0.182.0/streams/reader_from_iterable.ts';

function oggToMp3(file: ReadableStream<Uint8Array>): ReadableStream<Uint8Array> {
  const p = Deno.run({
    cmd: ['/usr/bin/ffmpeg', '-i', 'pipe:0', '-codec:a', 'libmp3lame', '-loglevel', 'error', '-f', 'mp3', 'pipe:1'],
    stdin: 'piped',
    stdout: 'piped',
    stderr: 'piped',
  });

  copy(readerFromIterable(file), p.stdin).catch((err) => console.warn(err)).finally(() => p.stdin.close());

  return p.stdout.readable;
}

function handleRequest(request: Request): Response {
  const url = new URL(request.url);

  if (url.pathname === '/') {
    return new Response('Hello World!');
  }

  if (url.pathname === '/ogg-to-mp3') {
    if (request.method !== 'POST') {
      return new Response('Method not allowed', { status: 405 });
    }

    if (request.headers.get('Content-Type') !== 'audio/ogg') {
      return new Response('Invalid content type', { status: 400 });
    }

    if (request.headers.get('Content-Length') === null) {
      return new Response('Missing content length', { status: 400 });
    }

    if (request.body === null) {
      return new Response('Missing request body', { status: 400 });
    }

    return new Response(oggToMp3(request.body), { headers: { 'Content-Type': 'audio/mpeg' } });
  }

  return new Response('Not found', { status: 404 });
}

async function handleConn(conn: Deno.Conn) {
  const httpConn = Deno.serveHttp(conn);

  for await (const { request, respondWith } of httpConn) {
    respondWith(handleRequest(request));
  }
}

const listener = Deno.listen({ port: 8080 });

console.log(`HTTP server running. Access it at: http://localhost:8080/`);

for await (const conn of listener) {
  handleConn(conn).catch((err) => console.warn(err));
}
