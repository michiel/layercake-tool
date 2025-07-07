#!/bin/bash

# Layercake Development Workflow Script
# This script starts both backend and frontend in development mode with hot reload

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BACKEND_PORT=3000
FRONTEND_PORT=3001
DB_PATH="./dev.db"

# Cleanup function
cleanup() {
    echo -e "\n${YELLOW}🛑 Shutting down development servers...${NC}"
    
    # Kill any running processes
    pkill -f "layercake serve" || true
    pkill -f "npm run dev" || true
    pkill -f "yarn dev" || true
    pkill -f "vite" || true
    
    echo -e "${GREEN}✅ Development servers stopped${NC}"
    exit 0
}

# Set up signal handlers
trap cleanup SIGINT SIGTERM

print_banner() {
    echo -e "${BLUE}"
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║                    🧅 Layercake Development                  ║"
    echo "║                                                              ║"
    echo "║  Backend:  http://localhost:$BACKEND_PORT                              ║"
    echo "║  Frontend: http://localhost:$FRONTEND_PORT                              ║"
    echo "║  Database: $DB_PATH                                    ║"
    echo "║                                                              ║"
    echo "║  Press Ctrl+C to stop all services                          ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

check_dependencies() {
    echo -e "${BLUE}🔍 Checking dependencies...${NC}"
    
    # Check if cargo is available
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}❌ Error: Cargo not found. Please install Rust.${NC}"
        exit 1
    fi
    
    # Check if Node.js is available (for future frontend)
    if ! command -v node &> /dev/null; then
        echo -e "${YELLOW}⚠️  Warning: Node.js not found. Frontend development will be limited.${NC}"
        echo -e "${YELLOW}   Install Node.js 18+ for full development experience.${NC}"
    fi
    
    echo -e "${GREEN}✅ Dependencies checked${NC}"
}

setup_database() {
    echo -e "${BLUE}🗄️  Setting up development database...${NC}"
    
    # Remove old database if exists
    if [ -f "$DB_PATH" ]; then
        echo -e "${YELLOW}🗑️  Removing old database: $DB_PATH${NC}"
        rm -f "$DB_PATH"
    fi
    
    echo -e "${GREEN}✅ Database setup complete${NC}"
}

start_backend() {
    echo -e "${BLUE}🚀 Starting Layercake backend...${NC}"
    
    # Start the backend with live reload features enabled
    cargo run -- serve \
        --port $BACKEND_PORT \
        --database "$DB_PATH" \
        --cors-origin "http://localhost:$FRONTEND_PORT" &
    
    BACKEND_PID=$!
    
    # Wait for backend to start
    echo -e "${YELLOW}⏳ Waiting for backend to start...${NC}"
    for i in {1..30}; do
        if curl -s http://localhost:$BACKEND_PORT/health > /dev/null 2>&1; then
            echo -e "${GREEN}✅ Backend started successfully on port $BACKEND_PORT${NC}"
            break
        fi
        
        if [ $i -eq 30 ]; then
            echo -e "${RED}❌ Backend failed to start after 30 seconds${NC}"
            cleanup
            exit 1
        fi
        
        sleep 1
    done
}

seed_database() {
    echo -e "${BLUE}🌱 Seeding development database with sample data...${NC}"
    
    # Seed the database with example projects and plans
    cargo run -- db seed --database "$DB_PATH"
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Database seeded successfully${NC}"
    else
        echo -e "${YELLOW}⚠️  Database seeding failed, continuing anyway${NC}"
    fi
}

setup_frontend() {
    echo -e "${BLUE}🎨 Setting up frontend development...${NC}"
    
    # Create frontend directory if it doesn't exist
    if [ ! -d "frontend" ]; then
        echo -e "${YELLOW}📁 Creating frontend directory structure...${NC}"
        mkdir -p frontend/{src,public,dist}
        
        # Create basic package.json
        cat > frontend/package.json << EOF
{
  "name": "layercake-frontend",
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  },
  "devDependencies": {
    "vite": "^5.0.0"
  },
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0"
  }
}
EOF
        
        # Create basic vite.config.js
        cat > frontend/vite.config.js << EOF
import { defineConfig } from 'vite'

export default defineConfig({
  server: {
    port: $FRONTEND_PORT,
    proxy: {
      '/api': 'http://localhost:$BACKEND_PORT',
      '/graphql': 'http://localhost:$BACKEND_PORT',
      '/mcp': 'http://localhost:$BACKEND_PORT',
      '/health': 'http://localhost:$BACKEND_PORT',
    }
  },
  build: {
    rollupOptions: {
      output: {
        entryFileNames: 'script.js',
        assetFileNames: (assetInfo) => {
          return assetInfo.name?.endsWith('.css') ? 'style.css' : '[name].[ext]';
        }
      }
    }
  }
})
EOF
        
        # Create basic index.html
        cat > frontend/index.html << EOF
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Layercake - Development</title>
</head>
<body>
    <div id="root">
        <div style="padding: 40px; text-align: center; font-family: system-ui, sans-serif;">
            <h1>🧅 Layercake Development</h1>
            <p>Frontend development server running</p>
            <div style="margin: 20px 0;">
                <a href="/health" style="margin: 10px; padding: 10px 20px; background: #007bff; color: white; text-decoration: none; border-radius: 4px;">Health Check</a>
                <a href="/docs" style="margin: 10px; padding: 10px 20px; background: #28a745; color: white; text-decoration: none; border-radius: 4px;">API Docs</a>
                <a href="/graphql" style="margin: 10px; padding: 10px 20px; background: #e91e63; color: white; text-decoration: none; border-radius: 4px;">GraphQL</a>
            </div>
            <details style="margin-top: 30px; text-align: left; max-width: 600px; margin-left: auto; margin-right: auto;">
                <summary style="cursor: pointer; font-weight: bold;">📋 Development Info</summary>
                <ul style="margin-top: 10px;">
                    <li><strong>Backend:</strong> http://localhost:$BACKEND_PORT</li>
                    <li><strong>Frontend:</strong> http://localhost:$FRONTEND_PORT</li>
                    <li><strong>Database:</strong> $DB_PATH</li>
                    <li><strong>Hot Reload:</strong> Enabled</li>
                </ul>
            </details>
        </div>
    </div>
    <script type="module" src="/src/main.js"></script>
</body>
</html>
EOF
        
        # Create basic main.js
        mkdir -p frontend/src
        cat > frontend/src/main.js << EOF
// Layercake Frontend Development Entry Point
console.log('🧅 Layercake frontend development server started');

// Test API connectivity
fetch('/health')
  .then(response => response.json())
  .then(data => {
    console.log('✅ Backend connectivity test successful:', data);
    
    // Update UI
    const root = document.getElementById('root');
    if (root) {
      const statusDiv = document.createElement('div');
      statusDiv.style.cssText = 'position: fixed; top: 10px; right: 10px; background: #d4edda; border: 1px solid #c3e6cb; color: #155724; padding: 8px 12px; border-radius: 4px; font-size: 12px;';
      statusDiv.textContent = '✅ Backend Connected';
      document.body.appendChild(statusDiv);
    }
  })
  .catch(error => {
    console.error('❌ Backend connectivity test failed:', error);
    
    // Update UI with error
    const root = document.getElementById('root');
    if (root) {
      const statusDiv = document.createElement('div');
      statusDiv.style.cssText = 'position: fixed; top: 10px; right: 10px; background: #f8d7da; border: 1px solid #f5c6cb; color: #721c24; padding: 8px 12px; border-radius: 4px; font-size: 12px;';
      statusDiv.textContent = '❌ Backend Disconnected';
      document.body.appendChild(statusDiv);
    }
  });
EOF
        
        echo -e "${GREEN}✅ Frontend structure created${NC}"
    fi
}

start_frontend() {
    # Check if Node.js is available
    if ! command -v node &> /dev/null; then
        echo -e "${YELLOW}⚠️  Node.js not available, skipping frontend dev server${NC}"
        echo -e "${YELLOW}   Frontend will be served from backend at http://localhost:$BACKEND_PORT${NC}"
        return
    fi
    
    cd frontend
    
    # Install dependencies if node_modules doesn't exist
    if [ ! -d "node_modules" ]; then
        echo -e "${BLUE}📦 Installing frontend dependencies...${NC}"
        if command -v npm &> /dev/null; then
            npm install
        elif command -v yarn &> /dev/null; then
            yarn install
        else
            echo -e "${RED}❌ No package manager found (npm or yarn)${NC}"
            cd ..
            return
        fi
    fi
    
    echo -e "${BLUE}🎨 Starting frontend development server...${NC}"
    
    # Start frontend dev server
    if command -v npm &> /dev/null; then
        npm run dev &
    elif command -v yarn &> /dev/null; then
        yarn dev &
    fi
    
    FRONTEND_PID=$!
    cd ..
    
    # Wait for frontend to start
    echo -e "${YELLOW}⏳ Waiting for frontend to start...${NC}"
    for i in {1..30}; do
        if curl -s http://localhost:$FRONTEND_PORT > /dev/null 2>&1; then
            echo -e "${GREEN}✅ Frontend started successfully on port $FRONTEND_PORT${NC}"
            break
        fi
        
        if [ $i -eq 30 ]; then
            echo -e "${YELLOW}⚠️  Frontend dev server not responding, using backend fallback${NC}"
            break
        fi
        
        sleep 1
    done
}

show_development_info() {
    echo
    echo -e "${GREEN}🎉 Development environment ready!${NC}"
    echo
    echo -e "${BLUE}📍 Available endpoints:${NC}"
    echo -e "  Frontend:    ${GREEN}http://localhost:$FRONTEND_PORT${NC}"
    echo -e "  Backend:     ${GREEN}http://localhost:$BACKEND_PORT${NC}"
    echo -e "  Health:      ${GREEN}http://localhost:$BACKEND_PORT/health${NC}"
    echo -e "  API Docs:    ${GREEN}http://localhost:$BACKEND_PORT/docs${NC}"
    echo -e "  GraphQL:     ${GREEN}http://localhost:$BACKEND_PORT/graphql${NC}"
    echo -e "  MCP:         ${GREEN}http://localhost:$BACKEND_PORT/mcp${NC}"
    echo
    echo -e "${BLUE}🛠️  Development features:${NC}"
    echo -e "  • Hot reload for backend changes (restart server)"
    echo -e "  • Hot reload for frontend changes (automatic)"
    echo -e "  • CORS configured for cross-origin requests"
    echo -e "  • API proxy from frontend to backend"
    echo -e "  • Development database with migrations"
    echo -e "  • Automatic database seeding with sample data"
    echo
    echo -e "${YELLOW}💡 Tips:${NC}"
    echo -e "  • Backend logs appear in this terminal"
    echo -e "  • Frontend changes auto-reload in browser"
    echo -e "  • Database is reset and reseeded on each restart"
    echo -e "  • Sample projects and plans are available immediately"
    echo -e "  • Press Ctrl+C to stop all services"
    echo
}

# Main execution
main() {
    print_banner
    check_dependencies
    setup_database
    start_backend
    seed_database
    setup_frontend
    start_frontend
    show_development_info
    
    # Keep script running and show logs
    echo -e "${BLUE}📄 Showing backend logs (press Ctrl+C to stop):${NC}"
    echo -e "${GREEN}=================================================${NC}"
    
    # Wait for backend process
    wait $BACKEND_PID
}

# Run main function
main "$@"