#!/bin/bash
set -euo pipefail

WITCHCRAFT_API_VERSION=2.6.0

curl \
    -LsSfo witchcraft-logging-api/witchcraft-logging-api.json \
    https://repo1.maven.org/maven2/com/palantir/witchcraft/api/witchcraft-logging-api/$WITCHCRAFT_API_VERSION/witchcraft-logging-api-$WITCHCRAFT_API_VERSION.conjure.json
