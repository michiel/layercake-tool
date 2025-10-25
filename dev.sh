#!/bin/bash

# Layercake Development Script
# Runs both frontend and backend in development mode
# Use --tauri flag to run in Tauri desktop mode

set -e

# Parse command line arguments
TAURI_MODE=false
while [[ $# -gt 0 ]]; do
    case $1 in
        --tauri)
            TAURI_MODE=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--tauri]"
            exit 1
            ;;
    esac
done

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
TAURI_DIR="src-tauri"
LOG_LEVEL="${LOG_LEVEL:-info}"

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

    # Kill background jobs
    if [[ -n $TAURI_PID ]]; then
        kill $TAURI_PID 2>/dev/null || true
    fi
    if [[ -n $BACKEND_PID ]]; then
        kill $BACKEND_PID 2>/dev/null || true
    fi
    if [[ -n $FRONTEND_PID ]]; then
        kill $FRONTEND_PID 2>/dev/null || true
    fi

    # Kill any remaining processes on our ports
    if [[ "$TAURI_MODE" == "false" ]]; then
        lsof -ti:$BACKEND_PORT | xargs kill -9 2>/dev/null || true
        lsof -ti:$FRONTEND_PORT | xargs kill -9 2>/dev/null || true
    fi

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

if [[ "$TAURI_MODE" == "false" ]]; then
    if [[ ! -d "$FRONTEND_DIR" ]]; then
        print_error "Frontend directory '$FRONTEND_DIR' not found"
        exit 1
    fi
else
    if [[ ! -d "$TAURI_DIR" ]]; then
        print_error "Tauri directory '$TAURI_DIR' not found"
        exit 1
    fi
    if [[ ! -d "$FRONTEND_DIR" ]]; then
        print_error "Frontend directory '$FRONTEND_DIR' not found (required for Tauri)"
        exit 1
    fi
fi

# Check if ports are available (only in web mode)
if [[ "$TAURI_MODE" == "false" ]]; then
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
fi

if [[ "$TAURI_MODE" == "true" ]]; then
    print_status "Starting Layercake in Tauri desktop mode..."
else
    print_status "Starting Layercake development environment..."
    print_status "Backend port: $BACKEND_PORT"
    print_status "Frontend port: $FRONTEND_PORT"
fi
print_status "Log level: $LOG_LEVEL"

# Initialize database if it doesn't exist
if [[ ! -f "layercake.db" ]]; then
    print_status "Initializing database..."
    cargo run --bin layercake -- db init
    print_success "Database initialized"
fi

# Branch based on mode
if [[ "$TAURI_MODE" == "true" ]]; then
    # ============================================
    # TAURI MODE: Run desktop application
    # ============================================
    print_status "Starting Tauri desktop application (this may take a moment to compile)..."

    # Tauri dev command handles both backend and frontend
    cargo tauri dev > tauri.log 2>&1 &
    TAURI_PID=$!

    # Wait for Tauri to start
    print_status "Waiting for Tauri to compile and start..."
    TAURI_WAIT=0
    TAURI_MAX_WAIT=120
    while [ $TAURI_WAIT -lt $TAURI_MAX_WAIT ]; do
        # Check if process is still alive
        if ! kill -0 $TAURI_PID 2>/dev/null; then
            print_error "Tauri process died. Check tauri.log for details:"
            tail -50 tauri.log
            exit 1
        fi

        # Check log for ready indicators
        if grep -q "Finished" tauri.log 2>/dev/null; then
            print_success "Tauri application started (PID: $TAURI_PID)"
            break
        fi

        sleep 3
        TAURI_WAIT=$((TAURI_WAIT + 3))

        if [ $TAURI_WAIT -ge $TAURI_MAX_WAIT ]; then
            print_error "Tauri failed to start within ${TAURI_MAX_WAIT}s. Check tauri.log for details:"
            tail -50 tauri.log
            exit 1
        fi
    done

    # Display connection info
    echo ""
    print_success "🚀 Layercake Tauri desktop application is ready!"
    echo ""
    echo -e "${YELLOW}📝 Log:${NC} tail -f tauri.log"
    echo ""
    echo -e "${GREEN}Press Ctrl+C to stop the application${NC}"

    # Monitor process
    while true; do
        sleep 30

        if ! kill -0 $TAURI_PID 2>/dev/null; then
            print_error "Tauri process died unexpectedly"
            exit 1
        fi

        print_status "Tauri application running (PID: $TAURI_PID)"
    done

else
    # ============================================
    # WEB MODE: Run separate backend and frontend
    # ============================================

    # Start backend server
    print_status "Starting backend server (this may take a moment to compile)..."
    cd "$BACKEND_DIR"
    cargo run --bin layercake -- serve --port $BACKEND_PORT --log-level $LOG_LEVEL --cors-origin "http://localhost:$FRONTEND_PORT" > backend.log 2>&1 &
    BACKEND_PID=$!
    cd - > /dev/null

    # Wait for backend to compile and start (with progress indicator)
    print_status "Waiting for backend to compile and start..."
    BACKEND_WAIT=0
    BACKEND_MAX_WAIT=60
    while [ $BACKEND_WAIT -lt $BACKEND_MAX_WAIT ]; do
        # Check if process is still alive
        if ! kill -0 $BACKEND_PID 2>/dev/null; then
            print_error "Backend process died. Check backend.log for details:"
            tail -30 backend.log
            exit 1
        fi

        # Check if server is responding
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

    # Check if node_modules exists, install if not
    if [[ ! -d "node_modules" ]]; then
        print_status "Installing frontend dependencies..."
        npm install
    fi

    # Update environment file for backend connection
    echo "VITE_API_BASE_URL=http://localhost:$BACKEND_PORT" > .env.development.local

    npm run dev -- --port $FRONTEND_PORT > ../frontend.log 2>&1 &
    FRONTEND_PID=$!
    cd - > /dev/null

    # Wait for frontend to start (with progress indicator)
    print_status "Waiting for frontend to start..."
    FRONTEND_WAIT=0
    FRONTEND_MAX_WAIT=30
    while [ $FRONTEND_WAIT -lt $FRONTEND_MAX_WAIT ]; do
        # Check if process is still alive
        if ! kill -0 $FRONTEND_PID 2>/dev/null; then
            print_error "Frontend process died. Check frontend.log for details:"
            tail -30 frontend.log
            exit 1
        fi

        # Check if server is responding (look for Vite's ready message in log)
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
    echo -e "${BLUE}📚 API Docs:${NC}        http://localhost:$BACKEND_PORT/swagger-ui/"
    echo -e "${BLUE}🔍 GraphQL:${NC}         http://localhost:$BACKEND_PORT/graphql"
    echo ""
    echo -e "${YELLOW}📝 Logs:${NC}"
    echo -e "   Backend: tail -f backend.log"
    echo -e "   Frontend: tail -f frontend.log"
    echo ""
    echo -e "${GREEN}Press Ctrl+C to stop all servers${NC}"

    # Function to check server health
    check_health() {
        backend_health=$(curl -s http://localhost:$BACKEND_PORT/health || echo "DOWN")
        frontend_health=$(curl -s http://localhost:$FRONTEND_PORT || echo "DOWN")

        if [[ "$backend_health" == "DOWN" ]] || [[ "$frontend_health" == "DOWN" ]]; then
            print_warning "Health check failed - one or more servers may be down"
        fi
    }

    # Monitor processes and show periodic status
    while true; do
        sleep 30

        # Check if processes are still running
        if ! kill -0 $BACKEND_PID 2>/dev/null; then
            print_error "Backend process died unexpectedly"
            exit 1
        fi

        if ! kill -0 $FRONTEND_PID 2>/dev/null; then
            print_error "Frontend process died unexpectedly"
            exit 1
        fi

        # Optional health check (uncomment to enable)
        # check_health

        print_status "Services running (Backend: $BACKEND_PID, Frontend: $FRONTEND_PID)"
    done
fi
