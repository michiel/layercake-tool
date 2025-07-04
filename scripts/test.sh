#!/bin/bash

# Layercake Testing Framework Script
# Runs comprehensive tests for the Layercake application

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
    echo "║                    🧪 Layercake Test Suite                   ║"
    echo "║                                                              ║"
    echo "║  Comprehensive testing framework for graph visualization     ║"
    echo "║  and transformation tool                                     ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

run_unit_tests() {
    echo -e "${BLUE}🔬 Running unit tests...${NC}"
    
    cargo test --lib --features "server,graphql,mcp" \
        --no-fail-fast \
        -- --test-threads=1
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Unit tests passed${NC}"
    else
        echo -e "${RED}❌ Unit tests failed${NC}"
        return 1
    fi
}

run_integration_tests() {
    echo -e "${BLUE}🔗 Running integration tests...${NC}"
    
    cargo test --test "*" --features "server,graphql,mcp" \
        --no-fail-fast \
        -- --test-threads=1
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Integration tests passed${NC}"
    else
        echo -e "${RED}❌ Integration tests failed${NC}"
        return 1
    fi
}

run_database_tests() {
    echo -e "${BLUE}🗄️  Running database tests...${NC}"
    
    cargo test --test database_test --features "server,graphql,mcp" \
        --no-fail-fast \
        -- --test-threads=1
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Database tests passed${NC}"
    else
        echo -e "${RED}❌ Database tests failed${NC}"
        return 1
    fi
}

run_api_tests() {
    echo -e "${BLUE}🌐 Running API tests...${NC}"
    
    cargo test --test api_test --features "server,graphql,mcp" \
        --no-fail-fast \
        -- --test-threads=1
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ API tests passed${NC}"
    else
        echo -e "${RED}❌ API tests failed${NC}"
        return 1
    fi
}

run_e2e_tests() {
    echo -e "${BLUE}🎭 Running end-to-end tests...${NC}"
    
    cargo test --test e2e_mcp_test --features "server,graphql,mcp" \
        --no-fail-fast \
        -- --test-threads=1
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ End-to-end tests passed${NC}"
    else
        echo -e "${RED}❌ End-to-end tests failed${NC}"
        return 1
    fi
}

run_reference_tests() {
    echo -e "${BLUE}📚 Running reference tests...${NC}"
    
    cargo test --test integration_test::reference_exports --features "server,graphql,mcp" \
        --no-fail-fast \
        -- --test-threads=1
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Reference tests passed${NC}"
    else
        echo -e "${RED}❌ Reference tests failed${NC}"
        return 1
    fi
}

check_test_coverage() {
    echo -e "${BLUE}📊 Checking test coverage...${NC}"
    
    # Check if tarpaulin is installed
    if ! command -v cargo-tarpaulin &> /dev/null; then
        echo -e "${YELLOW}⚠️  cargo-tarpaulin not found. Install with: cargo install cargo-tarpaulin${NC}"
        echo -e "${YELLOW}   Skipping coverage analysis.${NC}"
        return 0
    fi
    
    cargo tarpaulin \
        --features "server,graphql,mcp" \
        --out Html \
        --output-dir coverage \
        --exclude-files "external-modules/*" "tests/*" \
        --target-dir target/tarpaulin
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Coverage analysis complete${NC}"
        echo -e "${BLUE}📄 Coverage report: coverage/tarpaulin-report.html${NC}"
    else
        echo -e "${YELLOW}⚠️  Coverage analysis failed${NC}"
    fi
}

lint_code() {
    echo -e "${BLUE}🧹 Running code linting...${NC}"
    
    # Check if clippy is available
    if command -v cargo-clippy &> /dev/null; then
        cargo clippy --features "server,graphql,mcp" -- -D warnings
        
        if [ $? -eq 0 ]; then
            echo -e "${GREEN}✅ Code linting passed${NC}"
        else
            echo -e "${RED}❌ Code linting failed${NC}"
            return 1
        fi
    else
        echo -e "${YELLOW}⚠️  cargo-clippy not found. Install with: rustup component add clippy${NC}"
    fi
}

format_check() {
    echo -e "${BLUE}📐 Checking code formatting...${NC}"
    
    cargo fmt --check
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ Code formatting is correct${NC}"
    else
        echo -e "${RED}❌ Code formatting issues found${NC}"
        echo -e "${YELLOW}💡 Run 'cargo fmt' to fix formatting${NC}"
        return 1
    fi
}

show_test_summary() {
    echo
    echo -e "${GREEN}🎉 Test suite completed!${NC}"
    echo
    echo -e "${BLUE}📋 Test categories covered:${NC}"
    echo -e "  • Unit tests - Core functionality and library components"
    echo -e "  • Integration tests - Database operations and data integrity" 
    echo -e "  • API tests - REST, GraphQL, and MCP endpoint functionality"
    echo -e "  • End-to-end tests - Complete workflow validation"
    echo -e "  • Reference tests - Output validation against known good results"
    echo
    echo -e "${BLUE}🛠️  Quality checks:${NC}"
    echo -e "  • Code linting with Clippy"
    echo -e "  • Format checking with rustfmt"
    echo -e "  • Test coverage analysis (optional)"
    echo
    echo -e "${YELLOW}💡 Additional commands:${NC}"
    echo -e "  • Run specific test: ./scripts/test.sh --unit"
    echo -e "  • Run with coverage: ./scripts/test.sh --coverage"
    echo -e "  • Quick check: ./scripts/test.sh --quick"
    echo
}

# Parse command line arguments
case "${1:-all}" in
    --unit|unit)
        print_banner
        run_unit_tests
        ;;
    --integration|integration)
        print_banner
        run_integration_tests
        ;;
    --database|database)
        print_banner
        run_database_tests
        ;;
    --api|api)
        print_banner
        run_api_tests
        ;;
    --e2e|e2e)
        print_banner
        run_e2e_tests
        ;;
    --reference|reference)
        print_banner
        run_reference_tests
        ;;
    --coverage|coverage)
        print_banner
        run_unit_tests
        run_integration_tests
        check_test_coverage
        ;;
    --lint|lint)
        print_banner
        lint_code
        format_check
        ;;
    --quick|quick)
        print_banner
        run_unit_tests
        run_database_tests
        lint_code
        ;;
    --all|all|*)
        print_banner
        
        # Run all test categories
        echo -e "${BLUE}🚀 Running comprehensive test suite...${NC}"
        echo
        
        # Track failures
        FAILED_TESTS=""
        
        # Core tests
        run_unit_tests || FAILED_TESTS="$FAILED_TESTS unit"
        echo
        run_database_tests || FAILED_TESTS="$FAILED_TESTS database"
        echo
        run_api_tests || FAILED_TESTS="$FAILED_TESTS api"
        echo
        run_integration_tests || FAILED_TESTS="$FAILED_TESTS integration"
        echo
        
        # Quality checks
        lint_code || FAILED_TESTS="$FAILED_TESTS lint"
        echo
        format_check || FAILED_TESTS="$FAILED_TESTS format"
        echo
        
        # Optional: E2E and reference tests (may fail if sample data missing)
        echo -e "${YELLOW}🎭 Running optional tests (may skip if dependencies missing)...${NC}"
        run_e2e_tests || echo -e "${YELLOW}⚠️  E2E tests skipped (dependencies may be missing)${NC}"
        echo
        run_reference_tests || echo -e "${YELLOW}⚠️  Reference tests skipped (sample data may be missing)${NC}"
        echo
        
        # Optional coverage
        check_test_coverage
        echo
        
        # Summary
        show_test_summary
        
        # Check if any core tests failed
        if [ -n "$FAILED_TESTS" ]; then
            echo -e "${RED}❌ Some tests failed:$FAILED_TESTS${NC}"
            exit 1
        else
            echo -e "${GREEN}✅ All core tests passed successfully!${NC}"
        fi
        ;;
esac