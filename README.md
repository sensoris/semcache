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
