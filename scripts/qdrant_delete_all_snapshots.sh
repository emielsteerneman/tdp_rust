## To list all snapshots for a collection
# curl -X GET http://localhost:6333/collections/chunk/snapshots
# output example: {"result":[{"name":"chunk-4191152078487448-2026-02-15-09-56-42.snapshot","creation_time":"2026-02-15T09:56:42","size":295100416,"checksum":"a60c403dd27b2c164b1115f4e4c0166751e9a013fb234d3e0a433ea375f2c9af"},{"name":"chunk-4191152078487448-2026-02-15-10-08-23.snapshot","creation_time":"2026-02-15T10:08:23","size":295100416,"checksum":"a5ea1d0a888a8390ca39fbd18bd22eec58d519e35bff64f1d62c78a3a71b2815"},{"name":"chunk-4191152078487448-2026-02-15-10-04-15.snapshot","creation_time":"2026-02-15T10:04:15","size":295100416,"checksum":"15f7ec1cffd0041ee119c652ea151b6fa79a52df498b0abfc696fa8654692ed2"}],"status":"ok","time":0.001711893}

## To download the snapshot:
# curl -X GET http://localhost:6333/collections/chunk/snapshots/{snapshot_name} --output {output_file_path}

SNAPSHOTS=$(curl -X GET http://localhost:6333/collections/chunk/snapshots --no-progress-meter)
SNAPSHOT_NAMES=$(echo $SNAPSHOTS | jq -r '.result[].name')

if [ -z "$SNAPSHOT_NAMES" ]; then
    echo "No snapshots found for collection 'chunk'."
    exit 0
fi

echo "Found snapshots: $SNAPSHOT_NAMES"

for SNAPSHOT_NAME in $SNAPSHOT_NAMES; do
    echo "Deleting snapshot '$SNAPSHOT_NAME' from Qdrant..."
    CMD_DELETE=$(curl -X DELETE http://localhost:6333/collections/chunk/snapshots/${SNAPSHOT_NAME})
done