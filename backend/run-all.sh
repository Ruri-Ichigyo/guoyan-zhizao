#!/bin/bash

# Start all services in parallel for development
echo "Starting all services..."

cargo run --bin upload-svc &
UPLOAD_PID=$!

cargo run --bin query-svc &
QUERY_PID=$!

cargo run --bin sort-svc &
SORT_PID=$!

echo "Services started:"
echo "  Upload service (PID: $UPLOAD_PID) on port 3001"
echo "  Query service (PID: $QUERY_PID) on port 3002"
echo "  Sort service (PID: $SORT_PID)"
echo ""
echo "Press Ctrl+C to stop all services"

# Wait for Ctrl+C
trap "kill $UPLOAD_PID $QUERY_PID $SORT_PID; exit" INT
wait