### Local deployment

[Install and run Quickwit server](https://quickwit.io/docs/get-started/quickstart), then create indexes:

```bash
./quickwit index create --index-config .../eratosthenes-server/monitoring/quickwit/http_requests.yaml
```

```bash
./quickwit index create --index-config .../eratosthenes-server/monitoring/quickwit/client_sent_ws_messages.yaml
```

```bash
./quickwit index create --index-config .../eratosthenes-server/monitoring/quickwit/sockets_counts.yaml
```

Having thq Quickwit server up and running, launch the project like this:

```bash
cargo run -- --jwt-signing-key yourKeyHere
```

Or, run with Docker like this (see hwo to build the image below):

```bash
docker run --rm -p 3030:3030 eratosthenes-server --jwt-signing-key yourKeyHere
```

You can delete the indexes (say, to re-create them is schema changes) like this:

```bash
./quickwit index delete --index http_requests
```

```bash
./quickwit index delete --index client_sent_ws_messages
```

```bash
./quickwit index delete --index sockets_counts
```

### Public deployment

Install `gcloud` and add docker authentication for gcr.io as described here:

https://cloud.google.com/artifact-registry/docs/docker/authentication

Build, tag and push the docker image to the Google Cloud Registry:

```bash
docker build -t eratosthenes-server .
```

```bash
docker tag eratosthenes-server gcr.io/chess-project-44320/eratosthenes
```

```bash
docker push gcr.io/chess-project-44320/eratosthenes
```

Go to the Google Cloud Console and create a service from the image in the Google Cloud Registry.
Don't forget to pass in the `--quickwit-url` argument to the image entrypoint.
