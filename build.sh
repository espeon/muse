# get build (comma separated list of images to build, passed in as first argument)
build=$1

# if 'build' is "help" or build is none
if [ "$build" == "help" ]; then
    echo "build.sh <build> <tag>"
    echo "build: comma separated list of images to build"
    echo "  - valid values: nozomi, maki, umi, all"
    echo "tag: tag to build and push as"
    exit 0
fi

# if build is empty then exit
if [ -z "$build" ]; then
    echo "did not specify which images to build! run 'build.sh all' to build and push all images to latest"
    exit 1
fi

# get tag (passed in as second argument)
tag=$2

# if skip is not empty
if [ -n "$build" ]; then
    # split skip into an array
    IFS=',' read -r -a build_array <<< "$skip"
fi

# if tag is empty
if [ -z "$tag" ]; then
    echo "tag not specified, using latest"
    tag=latest
fi

# fancy title
echo "building and pushing images"
echo "---"

# build as 'ghcr.io/espeon/muse/nozomi:<tag>'
# if 'build' contains 'nozomi' or 'build' is 'all', build it
if [ "$build" == "nozomi" ] || [ "$build" == "all" ]; then
    echo "building nozomi:$tag"
    # build as 'ghcr.io/espeon/muse/nozomi:<tag>'
    docker buildx build --platform linux/amd64,linux/arm64 -t ghcr.io/espeon/muse/nozomi:$tag nozomi
fi

if [ "$build" == "maki" ] || [ "$build" == "all" ]; then
    echo "building maki:$tag"
    # build as 'ghcr.io/espeon/muse/maki:<tag>'
    docker buildx build --platform linux/amd64,linux/arm64 -t ghcr.io/espeon/muse/maki:$tag maki
fi

# check if 'umi' folder is present, also check if 'build' contains 'umi' or 'all'
if [ -d "umi" ] && ([ "$build" == "umi" ] || [ "$build" == "all" ]); then
    echo "building umi:$tag"
    docker buildx build --platform linux/amd64,linux/arm64 -t ghcr.io/espeon/umi/umi:$tag umi --push
fi

echo "done!"