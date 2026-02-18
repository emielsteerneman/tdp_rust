# https://qdrant.tech/documentation/concepts/collections/

# Check if collection 'chunk' exists
EXISTS_OUTPUT=$(curl -X GET http://localhost:6333/collections/chunk/exists --no-progress-meter)
EXISTS=$(echo $EXISTS_OUTPUT | jq -r '.result.exists')
if [ "$EXISTS" != "true" ]; then
    echo "Collection 'chunk' does not exist. Please create it before running this script."
    exit 1
fi

# Get collection stats
STATS_OUTPUT=$(curl -X GET http://localhost:6333/collections/chunk/ --no-progress-meter)

NCHUNKS=$(echo $STATS_OUTPUT | jq -r '.result.points_count')
VECSIZE=$(echo $STATS_OUTPUT | jq -r '.result.config.params.vectors.dense.size')

echo "Collection 'chunk' has $NCHUNKS chunks with vector size $VECSIZE."