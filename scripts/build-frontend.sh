#!/bin/bash

# Layercake Frontend Build Script
# Builds the React frontend and integrates it with the backend

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_banner() {
    echo -e "${BLUE}"
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║                 🏗️  Layercake Frontend Build                ║"
    echo "║                                                              ║"
    echo "║  Builds React frontend and integrates with backend          ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

check_dependencies() {
    echo -e "${BLUE}🔍 Checking build dependencies...${NC}"
    
    if ! command -v node &> /dev/null; then
        echo -e "${RED}❌ Error: Node.js not found. Please install Node.js 18+.${NC}"
        exit 1
    fi
    
    if [ ! -d "frontend" ]; then
        echo -e "${RED}❌ Error: Frontend directory not found.${NC}"
        exit 1
    fi
    
    if [ ! -f "frontend/package.json" ]; then
        echo -e "${RED}❌ Error: Frontend package.json not found.${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✅ Dependencies checked${NC}"
}

install_dependencies() {
    echo -e "${BLUE}📦 Installing frontend dependencies...${NC}"
    
    cd frontend
    
    if command -v npm &> /dev/null; then
        npm install
    elif command -v yarn &> /dev/null; then
        yarn install
    else
        echo -e "${RED}❌ Error: No package manager found (npm or yarn)${NC}"
        exit 1
    fi
    
    cd ..
    echo -e "${GREEN}✅ Dependencies installed${NC}"
}

build_frontend() {
    echo -e "${BLUE}🏗️  Building frontend for production...${NC}"
    
    cd frontend
    
    # Clean previous build
    if [ -d "dist" ]; then
        rm -rf dist
    fi
    
    # Build the frontend
    if command -v npm &> /dev/null; then
        npm run build
    elif command -v yarn &> /dev/null; then
        yarn build
    else
        echo -e "${RED}❌ Error: No package manager found${NC}"
        exit 1
    fi
    
    cd ..
    
    if [ ! -d "frontend/dist" ]; then
        echo -e "${RED}❌ Error: Frontend build failed - dist directory not found${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✅ Frontend built successfully${NC}"
}

integrate_with_backend() {
    echo -e "${BLUE}🔗 Integrating frontend with backend...${NC}"
    
    # Create backend static directory if it doesn't exist
    mkdir -p static
    
    # Copy built frontend files to backend static directory
    if [ -d "frontend/dist" ]; then
        echo -e "${YELLOW}📁 Copying frontend files to backend static directory...${NC}"
        cp -r frontend/dist/* static/
        echo -e "${GREEN}✅ Frontend files copied to static/${NC}"
    else
        echo -e "${RED}❌ Error: Frontend dist directory not found${NC}"
        exit 1
    fi
    
    # Verify integration
    if [ -f "static/index.html" ]; then
        echo -e "${GREEN}✅ Frontend successfully integrated with backend${NC}"
    else
        echo -e "${RED}❌ Error: Integration failed - index.html not found${NC}"
        exit 1
    fi
}

show_summary() {
    echo
    echo -e "${GREEN}🎉 Frontend build completed successfully!${NC}"
    echo
    echo -e "${BLUE}📋 Build summary:${NC}"
    echo -e "  • Frontend built in: frontend/dist/"
    echo -e "  • Static files copied to: static/"
    echo -e "  • Ready for production deployment"
    echo
    echo -e "${BLUE}🚀 Next steps:${NC}"
    echo -e "  • Run backend: cargo run --features server,graphql,mcp"
    echo -e "  • Frontend will be served from: http://localhost:3000/"
    echo -e "  • Static assets served from: /static/"
    echo
    echo -e "${YELLOW}💡 Development tip:${NC}"
    echo -e "  • Use ./scripts/dev.sh for development with hot reload"
    echo -e "  • Use this script for production builds"
    echo
}

# Parse command line arguments
case "${1:-build}" in
    --help|help)
        print_banner
        echo "Usage: $0 [command]"
        echo ""
        echo "Commands:"
        echo "  build (default)  - Build frontend and integrate with backend"
        echo "  --help          - Show this help message"
        echo ""
        echo "This script builds the React frontend for production and"
        echo "copies the built files to the backend's static directory."
        ;;
    --build|build|*)
        print_banner
        check_dependencies
        install_dependencies
        build_frontend
        integrate_with_backend
        show_summary
        ;;
esac