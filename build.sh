VERSION="0.1"

# Parse arguments
for i in "$@"; do
  case "$1" in
    -d | --docker ) DOCKER=true ;;
    -v | --version) VERSION="$2"; shift ;;
    * ) break ;;
  esac
  shift
done

# Build Docker image?
if [ $DOCKER ]; then
    docker build -t "duckhq/duck:latest" \
        -t "duckhq/duck:$VERSION" \
        -t "spectresystems/duck:latest" \
        -t "spectresystems/duck:$VERSION" \
        --build-arg "VERSION=$VERSION" .
fi