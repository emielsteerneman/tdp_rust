#!/bin/sh
/app/web &
BACKEND_PID=$!
cd /app/frontend && npm run dev -- --host &
FRONTEND_PID=$!
trap "kill $BACKEND_PID $FRONTEND_PID" EXIT TERM INT
wait
