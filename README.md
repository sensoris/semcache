## Call service

```shell
curl http://localhost:8080/chat/completions \
 -H "host: api.openai.com" \
 -H "Content-Type: application/json" \
 -H "Authorization: Bearer $OPENAI_API_KEY" \
 -H "X-LLM-Proxy-Upstream: https://api.openai.com/v1/chat/completions" \
 -d '{ "model": "gpt-4o", "messages": [{"role": "user", "content": "Hello?"}]}'
```

## Building faiss

Following commands should be executed in faiss root

### Generate build files

```shell
cmake -B build \
  -DFAISS_ENABLE_PYTHON=OFF \
  -DFAISS_ENABLE_GPU=OFF \
  -DBUILD_TESTING=OFF \
  -DCMAKE_BUILD_TYPE=Release \
  -DFAISS_ENABLE_C_API=ON \
  -DBUILD_SHARED_LIBS=ON \
  .
```

### build the artifact

```shell
cmake --build build -j$(nproc)
```

### install onto your system

```shell
cmake --install build
```

## Docker

The Dockerfile uses the base image created using `docker/Dockerfile.faiss`

Build:

```shell
docker build -f Dockerfile -t semcache-rs .
```

Running:

```shell
docker run -p 8080:8080 semcache-rs
```

Running with your own configuration:

```shell
docker run -p 8080:8080 --env-file my_env_file.env semcache-rs
```

For an example of how your env could look, have a look at docker/example.env
In order to see which configuration values are currently supported, look at the config.yaml file. An environment variable maps to a configuration key by stripping the SEMCACHE\_ prefix, and converting the remaining string to lowercase.

So in order to override the "port" configuration, define SEMCACHE_PORT in your environment file.

You can also override single variables using the -e flag.

## Scripts

Easy way to send a request:

```shell
➜ python scripts/request.py openai $API_KEY "What is the capital of France?"

⏱️  Request completed in 894ms
==================================================
The capital of France is Paris.
```
