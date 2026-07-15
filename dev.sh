#!/bin/bash

# Layercake Development Script
# Runs the backend (Rust) and the frontend (Vite dev server) together.

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BACKEND_PORT=3001
FRONTEND_PORT=1422
BACKEND_DIR="."
FRONTEND_DIR="frontend"
PROJECTIONS_DIR="projections-frontend"
LOG_LEVEL="${LOG_LEVEL:-debug}"
LOCAL_AUTH_BYPASS="${LAYERCAKE_LOCAL_AUTH_BYPASS:-1}"

# Function to print colored output
print_status() {
    echo -e "${BLUE}[DEV]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to cleanup background processes
cleanup() {
    print_status "Shutting down development servers..."

    if [[ -n $BACKEND_PID ]]; then
        kill $BACKEND_PID 2>/dev/null || true
    fi
    if [[ -n $FRONTEND_PID ]]; then
        kill $FRONTEND_PID 2>/dev/null || true
    fi

    # Kill any remaining processes on our ports
    lsof -ti:$BACKEND_PORT | xargs kill -9 2>/dev/null || true
    lsof -ti:$FRONTEND_PORT | xargs kill -9 2>/dev/null || true

    print_success "Development servers stopped"
    exit 0
}

# Set up signal handlers
trap cleanup SIGINT SIGTERM EXIT

# Check if required directories exist
if [[ ! -d "$BACKEND_DIR" ]]; then
    print_error "Backend directory '$BACKEND_DIR' not found"
    exit 1
fi

if [[ ! -d "$FRONTEND_DIR" ]]; then
    print_error "Frontend directory '$FRONTEND_DIR' not found"
    exit 1
fi

# Check if ports are available
if lsof -i:$BACKEND_PORT >/dev/null 2>&1; then
    print_warning "Port $BACKEND_PORT is already in use. Attempting to free it..."
    lsof -ti:$BACKEND_PORT | xargs kill -9 2>/dev/null || true
    sleep 2
fi

if lsof -i:$FRONTEND_PORT >/dev/null 2>&1; then
    print_warning "Port $FRONTEND_PORT is already in use. Attempting to free it..."
    lsof -ti:$FRONTEND_PORT | xargs kill -9 2>/dev/null || true
    sleep 2
fi

# Build projection viewer assets so backend can serve /projections/viewer/*
ensure_projection_viewer_build() {
    if [[ ! -d "$PROJECTIONS_DIR" ]]; then
        print_warning "Projection viewer directory '$PROJECTIONS_DIR' not found; skipping build"
        return
    fi

    print_status "Building projection viewer assets..."
    pushd "$PROJECTIONS_DIR" > /dev/null

    if [[ ! -d "node_modules" ]]; then
        print_status "Installing projection viewer dependencies..."
        npm install
    fi

    echo "VITE_API_BASE_URL=http://localhost:$BACKEND_PORT" > .env.local
    npm run build

    popd > /dev/null
    print_success "Projection viewer build ready"
}

print_status "Starting Layercake development environment..."
print_status "Backend port: $BACKEND_PORT"
print_status "Frontend port: $FRONTEND_PORT"
print_status "Log level: $LOG_LEVEL"
print_status "Local auth bypass: $LOCAL_AUTH_BYPASS"

# Initialize database if it doesn't exist
if [[ ! -f "layercake.db" ]]; then
    print_status "Initializing database..."
    cargo run --bin layercake -- db init
    print_success "Database initialized"
fi

# The main frontend runs via the Vite dev server (HMR), so it is NOT prebuilt
# here. Only the projection viewer needs a static build for the backend to serve.
ensure_projection_viewer_build

# Start backend server
print_status "Starting backend server (this may take a moment to compile)..."
cd "$BACKEND_DIR"
LAYERCAKE_LOCAL_AUTH_BYPASS="$LOCAL_AUTH_BYPASS" \
  cargo run --bin layercake -- serve --port $BACKEND_PORT --log-level $LOG_LEVEL --cors-origin "http://localhost:$FRONTEND_PORT" > backend.log 2>&1 &
BACKEND_PID=$!
cd - > /dev/null

# Wait for backend to compile and start (with progress indicator)
print_status "Waiting for backend to compile and start..."
BACKEND_WAIT=0
BACKEND_MAX_WAIT=360
while [ $BACKEND_WAIT -lt $BACKEND_MAX_WAIT ]; do
    if ! kill -0 $BACKEND_PID 2>/dev/null; then
        print_error "Backend process died. Check backend.log for details:"
        tail -30 backend.log
        exit 1
    fi

    if curl -s -f http://localhost:$BACKEND_PORT/health > /dev/null 2>&1; then
        print_success "Backend server started and responding (PID: $BACKEND_PID)"
        break
    fi

    sleep 2
    BACKEND_WAIT=$((BACKEND_WAIT + 2))

    if [ $BACKEND_WAIT -ge $BACKEND_MAX_WAIT ]; then
        print_error "Backend failed to start within ${BACKEND_MAX_WAIT}s. Check backend.log for details:"
        tail -30 backend.log
        exit 1
    fi
done

# Start frontend server
print_status "Starting frontend server..."
cd "$FRONTEND_DIR"

if [[ ! -d "node_modules" ]]; then
    print_status "Installing frontend dependencies..."
    npm install
fi

# Point the dev server at the backend for same-origin-style proxying.
echo "VITE_API_BASE_URL=http://localhost:$BACKEND_PORT" > .env.development.local

npm run dev -- --port $FRONTEND_PORT > ../frontend.log 2>&1 &
FRONTEND_PID=$!
cd - > /dev/null

# Wait for frontend to start (with progress indicator)
print_status "Waiting for frontend to start..."
FRONTEND_WAIT=0
FRONTEND_MAX_WAIT=30
while [ $FRONTEND_WAIT -lt $FRONTEND_MAX_WAIT ]; do
    if ! kill -0 $FRONTEND_PID 2>/dev/null; then
        print_error "Frontend process died. Check frontend.log for details:"
        tail -30 frontend.log
        exit 1
    fi

    if grep -q "Local:.*localhost:$FRONTEND_PORT" frontend.log 2>/dev/null; then
        print_success "Frontend server started and responding (PID: $FRONTEND_PID)"
        break
    fi

    sleep 2
    FRONTEND_WAIT=$((FRONTEND_WAIT + 2))

    if [ $FRONTEND_WAIT -ge $FRONTEND_MAX_WAIT ]; then
        print_error "Frontend failed to start within ${FRONTEND_MAX_WAIT}s. Check frontend.log for details:"
        tail -30 frontend.log
        exit 1
    fi
done

# Display connection info
echo ""
print_success "🚀 Layercake development environment is ready!"
echo ""
echo -e "${BLUE}📊 Backend API:${NC}     http://localhost:$BACKEND_PORT"
echo -e "${BLUE}🌐 Frontend App:${NC}    http://localhost:$FRONTEND_PORT"
echo -e "${BLUE}🔍 GraphQL:${NC}         http://localhost:$BACKEND_PORT/graphql"
echo ""
echo -e "${YELLOW}📝 Logs:${NC}"
echo -e "   Backend: tail -f backend.log"
echo -e "   Frontend: tail -f frontend.log"
echo ""
echo -e "${GREEN}Press Ctrl+C to stop all servers${NC}"

# Monitor processes and show periodic status
while true; do
    sleep 30

    if ! kill -0 $BACKEND_PID 2>/dev/null; then
        print_error "Backend process died unexpectedly"
        exit 1
    fi

    if ! kill -0 $FRONTEND_PID 2>/dev/null; then
        print_error "Frontend process died unexpectedly"
        exit 1
    fi

    print_status "Services running (Backend: $BACKEND_PID, Frontend: $FRONTEND_PID)"
done
