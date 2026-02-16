# https://qdrant.tech/documentation/concepts/collections/
# https://qdrant.tech/documentation/concepts/snapshots/

# Check if collection 'chunk' exists
EXISTS_OUTPUT=$(curl -X GET http://localhost:6333/collections/chunk/exists --no-progress-meter)
EXISTS=$(echo $EXISTS_OUTPUT | jq -r '.result.exists')
if [ "$EXISTS" != "true" ]; then
    echo "Collection 'chunk' does not exist. Please create it before running this script."
    exit 1
fi

# Get collection stats
STATS_OUTPUT=$(curl -X GET http://localhost:6333/collections/chunk/ --no-progress-meter)
echo $STATS_OUTPUT

NCHUNKS=$(echo $STATS_OUTPUT | jq -r '.result.points_count')
VECSIZE=$(echo $STATS_OUTPUT | jq -r '.result.config.params.vectors.dense.size')

echo "Collection 'chunk' has $NCHUNKS chunks with vector size $VECSIZE."

# Create snapshot
echo "Creating snapshot for collection 'chunk'..."

CREATE_OUTPUT=$(curl -X POST http://localhost:6333/collections/chunk/snapshots --no-progress-meter)
FILENAME=$(echo $CREATE_OUTPUT | jq -r '.result.name')

# Download snapshot
echo "Downloading snapshot '$FILENAME' to 'qdrant_${NCHUNKS}x${VECSIZE}.snapshot'..."

CMD_DOWNLOAD=$(curl -X GET http://localhost:6333/collections/chunk/snapshots/${FILENAME} --output qdrant_${NCHUNKS}x${VECSIZE}.snapshot --no-progress-meter)

# Delete snapshot
echo "Deleting snapshot '$FILENAME' from Qdrant..."

CMD_DELETE=$(curl -X DELETE http://localhost:6333/collections/chunk/snapshots/${FILENAME} --no-progress-meter)