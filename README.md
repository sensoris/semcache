## Docker build

`docker build -t semcache .`

## Docker run

`docker run --rm -p 8000:8000 --memory="128m" semcache`

## Call service

```shell
curl http://localhost:8000/chat/completions \
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
