docker buildx build --platform linux/amd64,linux/arm64 -t ghcr.io/espeon/muse/nozomi:latest nozomi --push
docker buildx build --platform linux/amd64,linux/arm64 -t ghcr.io/espeon/muse/maki:latest maki --push

# check if 'umi' folder is present
if [ -d "umi" ]; then
    docker buildx build --platform linux/amd64,linux/arm64 -t ghcr.io/espeon/umi/umi:latest umi --push
fi