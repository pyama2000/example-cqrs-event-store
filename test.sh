#!/usr/bin/env bash

set -eu -o pipefail

WIDGET_ID=$(
  curl -s http://localhost:8080/widgets \
    -H "Content-Type: application/json" \
    -d '
      {
        "widget_name": "widget name 1",
        "widget_description": "widget description 1"
      }' \
    | jq -r '.widget_id'
)
curl -v "http://localhost:8080/widgets/${WIDGET_ID}/name" \
  -H "Content-Type: application/json" \
  -d '{ "widget_name": "widget name 2"}'
curl -v "http://localhost:8080/widgets/${WIDGET_ID}/description" \
  -H "Content-Type: application/json" \
  -d '{ "widget_description": "widget description 2"}'
