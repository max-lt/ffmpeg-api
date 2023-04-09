import { copy } from 'https://deno.land/std@0.182.0/streams/copy.ts';
import { readerFromIterable } from 'https://deno.land/std@0.182.0/streams/reader_from_iterable.ts';

type OutFormat = 'mp3' | 'wav';

function convertOgg(format: OutFormat, input: ReadableStream<Uint8Array>): ReadableStream<Uint8Array> {
  const p = Deno.run({
    cmd: ['/usr/bin/ffmpeg', '-i', 'pipe:0', '-codec:a', 'libmp3lame', '-loglevel', 'error', '-f', format, 'pipe:1'],
    stdin: 'piped',
    stdout: 'piped',
    stderr: 'piped'
  });

  copy(readerFromIterable(input), p.stdin)
    .catch((err) => console.warn(`Error while copying to stdin:`, err))
    .finally(() => p.stdin.close());

  return p.stdout.readable;
}

function handleRequest(request: Request): Response {
  const url = new URL(request.url);

  if (url.pathname === '/') {
    return new Response('Hello World!');
  }

  if (url.pathname === '/ogg-to-mp3' || url.pathname === '/ogg-to-wav') {
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

    const format = url.pathname.slice(-3) as OutFormat;
    const outType = format === 'mp3' ? 'audio/mpeg' : 'audio/wav';

    return new Response(convertOgg(format, request.body), { headers: { 'Content-Type': outType } });
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
