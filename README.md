### Public deployment

Install `gcloud` and add docker authentication for gcr.io as described here:

https://cloud.google.com/artifact-registry/docs/docker/authentication

Build, tag and push the docker image to the Google Cloud Registry:

```bash
docker build -t eratosthenes-server . && docker image prune --filter label=stage=builder
```

```bash
docker tag eratosthenes-server gcr.io/chess-project-44320/eratosthenes
```

```bash
docker push gcr.io/chess-project-44320/eratosthenes
```

Go to the Google Cloud Console and create a service from the image in the Google Cloud Registry.
