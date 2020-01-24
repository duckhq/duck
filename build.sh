# Parse arguments
for i in "$@"; do
  case "$1" in
    -d | --docker ) DOCKER=true ;;
    -d | --schema ) SCHEMA=true ;;
    -v | --version) VERSION="$2"; shift ;;
    * ) break ;;
  esac
  shift
done

# Generate the JSON schema?
if [ $SCHEMA ]; then
  if [ ! -z $VERSION ]; then
    cargo run -- schema > "./schemas/v$VERSION.json"
  else
    echo "You must specify a version using the --version option."
    exit -1;
  fi
fi

# Build Docker image?
if [ $DOCKER ]; then
    docker build -t "duckhq/duck:latest" \
        -t "duckhq/duck:$VERSION" \
        -t "spectresystems/duck:latest" \
        -t "spectresystems/duck:$VERSION" \
        --build-arg "VERSION=$VERSION" .
fi