Backend server for a [geoguesser-like web game](https://github.com/v-spassky/eratosthenes-client).

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

Set up local S3-compatible object storage, for example [Localstack](https://docs.localstack.cloud/user-guide/aws/s3/):

```bash
localstack start
```

```bash
awslocal s3api create-bucket --bucket ert-chat-message-images --region eu-west-1 --create-bucket-configuration LocationConstraint=eu-west-1
```

Set AWS credentials and settings before running the

```bash
export AWS_ACCESS_KEY_ID=test && \
export AWS_SECRET_ACCESS_KEY=test && \
export AWS_REGION=eu-west-1 && \
export AWS_ENDPOINT_URL=http://localhost:4566 && \
export S3_FORCE_PATH_STYLE=true
```

Having the Quickwit server up and running, launch the project like this:

```bash
cargo run -- --jwt-signing-key yourKeyHere
```

Or, run with Docker like this (see how to build the image below):

```bash
docker run \
    --network host \ # if Localstack, Quickwit and other satellite infrastructure runs on your host machine
    --env AWS_ACCESS_KEY_ID=someAccessKeyId \ # can be anything if using Localstack instead of actual AWS S3 bucket
    --env AWS_SECRET_ACCESS_KEY=someAccessKey \ # can be anything if using Localstack instead of actual AWS S3 bucket
    --env AWS_REGION=s3-bucket-region \
    --env AWS_ENDPOINT_URL=http://localhost:4566 \ # if using Localstack instead of actual AWS S3 bucket
    --env S3_FORCE_PATH_STYLE=true \ # if using Localstack instead of actual AWS S3 bucket
    eratosthenes-server \
    --jwt-signing-key yourKeyHere
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
docker tag eratosthenes-server gcr.io/project-id/repo-id
```

```bash
docker push gcr.io/project-id/repo-id
```

Go to the Google Cloud Console and create a service from the image in the Google Cloud Registry.
Don't forget to pass in the `--quickwit-url` argument to the image entrypoint.
