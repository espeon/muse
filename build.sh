# get build (comma separated list of images to build, passed in as first argument)
build=$1

# valid values
valid_builds="nozomi,maki"

# add ',umi' if we can find the umi folder
if [ -d "umi" ]; then
    valid_builds="$valid_builds,umi"
fi

# if 'build' is "help" or build is none
if [ "$build" == "help" ]; then
    echo "build.sh <build> <tag>"
    echo "build: comma separated list of images to build"
    echo "  - valid values: $valid_builds, all"
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


# if build is not empty
if [ -n "$build" ]; then
    # split skip into an array
    IFS=',' read -r -a build_array <<< "$build"
fi

# if tag is empty
if [ -z "$tag" ]; then
    echo "tag not specified, using latest"
    tag=latest
fi

# fancy title
echo "building and pushing images"
echo "---"

# remove umi from valid builds, will be handled separately
valid_builds=$(echo $valid_builds | sed 's/,umi//g')

# build as 'ghcr.io/espeon/muse/<service>:<tag>'
for i in $(echo $valid_builds | sed "s/,/ /g")
do
    # if build contains value or build is 'all'
    if [[ $(echo ${build_array[@]} | fgrep -w $i) ]] || [ "$build" == "all" ]; then
        echo "building $i:$tag"
        #docker buildx build --platform linux/amd64,linux/arm64 -t ghcr.io/espeon/muse/nozomi:$tag nozomi
    fi
done

# check if 'umi' folder is present, also check if 'build' contains 'umi' or 'all'
if [ -d "umi" ] && ( [[ $(echo ${build_array[@]} | fgrep -w umi) ]] || [ "$build" == "all" ]); then
    echo "building umi:$tag"
    #docker buildx build --platform linux/amd64,linux/arm64 -t ghcr.io/espeon/umi/umi:$tag umi --push
fi

echo "done!"