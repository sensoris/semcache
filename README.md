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
sudo cmake --install build
```

## Building with docker

Building the project with docker is a two stage process, since we need a docker container with faiss installed locally onto the system.

### Build a base image with faiss 

#### Note: name is important as this is referenced in downstream Dockerfiles

```shell
sudo docker build -f Dockerfile.faiss -t faiss-base-image
```

### Build the release version of semcache

```shell
sudo docker build -f Dockerfile -t semcache-rs .
```

### Build a container for running tests

```shell
sudo docker build -f Dockerfile.test -t semcache-test .
```

## Scripts

Easy way to send request:

```shell
➜ python scripts/request.py openai $API_KEY "What is the capital of France?"

⏱️  Request completed in 894ms
==================================================
The capital of France is Paris.
```
