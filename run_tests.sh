#!/bin/bash

# End-to-end test runner for FeedTape Backend

set -e

echo "🧪 Running FeedTape Backend E2E Tests"
echo "======================================"
echo ""

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker and try again."
    exit 1
fi

echo "✅ Docker is running"
echo ""

# Parse command line arguments
TEST_FILTER=""
VERBOSE=""
RELEASE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --test)
            TEST_FILTER="--test $2"
            shift 2
            ;;
        --verbose)
            VERBOSE="-- --nocapture"
            shift
            ;;
        --release)
            RELEASE="--release"
            shift
            ;;
        --help)
            echo "Usage: ./run_tests.sh [options]"
            echo ""
            echo "Options:"
            echo "  --test <name>   Run specific test file (e.g., test_feeds)"
            echo "  --verbose       Show test output"
            echo "  --release       Run tests in release mode (faster)"
            echo "  --help          Show this help message"
            echo ""
            echo "Examples:"
            echo "  ./run_tests.sh                     # Run all tests"
            echo "  ./run_tests.sh --test test_feeds   # Run feed tests only"
            echo "  ./run_tests.sh --verbose           # Run with output"
            echo "  ./run_tests.sh --release --verbose # Fast mode with output"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Set up environment
export RUST_BACKTRACE=1
export RUST_LOG=warn,feedtape_backend=debug

# Clean up any hanging containers from previous runs
echo "🧹 Cleaning up previous test containers..."
docker ps -a | grep testcontainers | awk '{print $1}' | xargs -r docker rm -f 2>/dev/null || true

# Run the tests
echo "🚀 Starting tests..."
echo ""

if [ -n "$TEST_FILTER" ]; then
    echo "Running specific test: $TEST_FILTER"
else
    echo "Running all E2E tests..."
fi

cargo test $RELEASE $TEST_FILTER $VERBOSE

# Check exit code
if [ $? -eq 0 ]; then
    echo ""
    echo "✅ All tests passed!"
else
    echo ""
    echo "❌ Some tests failed. See output above for details."
    exit 1
fi