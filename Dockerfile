# ffmpeg-api image
FROM denoland/deno:alpine

RUN apk add --no-cache ffmpeg nano

RUN mkdir -p /build/ffmpeg-api/dist

WORKDIR /build/ffmpeg-api

COPY . /build/ffmpeg-api

# Prefer not to run as root.
USER deno

# Preload lib
RUN deno eval "import { copy } from 'https://deno.land/std@0.182.0/streams/copy.ts'; console.log('Loaded copy lib', void copy)"
RUN deno eval "import { readerFromIterable } from 'https://deno.land/std@0.182.0/streams/reader_from_iterable.ts'; console.log('Loaded reader_from_iterable lib', void readerFromIterable)"

CMD [ "deno", "run", "--allow-net", "--allow-run=/usr/bin/ffmpeg", "src/main.ts" ]
