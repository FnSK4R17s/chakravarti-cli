#!/bin/bash
set -e

# Chakravarti Executor Entrypoint
# Starts the worker process to consume tasks from Redis queue

echo "🚀 Starting Chakravarti Executor"
echo "   Agent: $AGENT_NAME"
echo "   Role: $AGENT_ROLE"
echo "   Provider: $AGENT_PROVIDER"
echo ""

# Configure git for this agent
git config --global user.name "Chakravarti $AGENT_NAME"
git config --global user.email "$AGENT_NAME@chakravarti.local"

# Wait for Redis to be available
REDIS_HOST="${REDIS_HOST:-chakravarti-redis}"
REDIS_PORT="${REDIS_PORT:-6379}"

echo "⏳ Waiting for Redis ($REDIS_HOST:$REDIS_PORT)..."
timeout=30
while ! nc -z "$REDIS_HOST" "$REDIS_PORT" 2>/dev/null; do
    timeout=$((timeout - 1))
    if [ $timeout -le 0 ]; then
        echo "❌ Redis not available after 30 seconds"
        exit 1
    fi
    sleep 1
done
echo "✓ Redis is available"

# Start the worker
echo ""
echo "🔄 Starting worker for queue: executor-$AGENT_NAME"
echo "   Listening for tasks..."
echo ""

# Run the worker (using the installed ckrv CLI)
exec ckrv worker -e "$AGENT_NAME"
