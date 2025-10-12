# Development Scripts

This directory contains scripts to run the Layercake development environment with both frontend and backend services.

## Quick Start

### Linux/macOS
```bash
./dev.sh
```

### Windows
```cmd
dev.bat
```

## What the Scripts Do

1. **Check Prerequisites**: Verify that required directories exist
2. **Port Management**: Free up ports 8080 (backend) and 1420 (frontend) if occupied
3. **Database Setup**: Initialize the SQLite database if it doesn't exist
4. **Backend Server**: Start the Rust backend with GraphQL and REST APIs
5. **Frontend Server**: Start the React frontend with hot reload
6. **Environment Setup**: Configure CORS and API endpoints automatically
7. **Health Monitoring**: Monitor both services and provide status updates

## Configuration

You can customize the setup using environment variables:

```bash
# Set log level (debug, info, warn, error)
export LOG_LEVEL=debug

# Custom ports (if needed)
export BACKEND_PORT=3000
export FRONTEND_PORT=5173

./dev.sh
```

## Services & URLs

When running, the following services will be available:

| Service | URL | Description |
|---------|-----|-------------|
| **Frontend App** | http://localhost:1420 | React application with Plan DAG editor |
| **Backend API** | http://localhost:8080 | REST and GraphQL APIs |
| **API Documentation** | http://localhost:8080/swagger-ui/ | Swagger UI for REST APIs |
| **GraphQL Playground** | http://localhost:8080/graphql | GraphQL endpoint for queries/mutations |

## Log Files

The scripts create log files in the project root:

- `backend.log` - Backend server logs
- `frontend.log` - Frontend build and dev server logs

View logs in real-time:
```bash
# Backend logs
tail -f backend.log

# Frontend logs
tail -f frontend.log
```

## Stopping Services

- **Graceful**: Press `Ctrl+C` in the terminal running the script
- **Force**: The script automatically cleans up ports on exit

## Troubleshooting

### Port Already in Use
The script automatically attempts to free ports, but if issues persist:

```bash
# Kill processes on specific ports
lsof -ti:8080 | xargs kill -9
lsof -ti:1420 | xargs kill -9
```

### Database Issues
If database initialization fails:

```bash
# Manually initialize
cd layercake-core
cargo run -- db init
```

### Frontend Dependencies
If frontend fails to start:

```bash
# Clean install
cd frontend
rm -rf node_modules package-lock.json
npm install
```

### Backend Compilation
If backend fails to compile:

```bash
# Clean build
cd layercake-core
cargo clean
cargo build
```

## Development Workflow

1. **Start Development**: Run `./dev.sh` (or `dev.bat` on Windows)
2. **Open Frontend**: Navigate to http://localhost:1420
3. **Test Plan Editor**: Click "Plan Editor" in the sidebar
4. **Make Changes**: Edit code - both frontend and backend support hot reload
5. **View Logs**: Monitor `backend.log` and `frontend.log` for issues
6. **Stop Services**: Press `Ctrl+C` when done

## Features in Development Mode

- **Hot Reload**: Frontend automatically reloads on code changes
- **CORS Enabled**: Backend configured to accept requests from frontend
- **Database Persistence**: SQLite database persists between runs
- **Error Reporting**: Detailed error logs for debugging
- **Health Monitoring**: Automatic detection of service failures

## Phase 1.3 Status

✅ **Implemented**:
- ReactFlow Plan DAG visual editor
- Apollo Client with GraphQL integration
- Custom node types and connection validation
- Real-time collaboration framework
- Professional UI with navigation

🔄 **In Progress**:
- Backend GraphQL API implementation
- Database schema for Plan DAG storage
- Node configuration dialogs

The development scripts are ready to support the complete Phase 1.3 frontend and future backend development.