docker buildx create --name multiplatform --driver docker-container --use

docker buildx inspect --bootstrap

docker buildx build --platform linux/amd64,linux/arm64 \
  -f Dockerfile.faiss \
  -t ghcr.io/sensoris/faiss-base-image:latest \
  --push .