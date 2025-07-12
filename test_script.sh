#!/bin/bash

# Comprehensive Test Suite for Mesh Generator
# Tests various algorithms, parameters, and geometries
# Saves results to log file with progress tracking

# Configuration
LOG_FILE="test_results_$(date +%Y%m%d_%H%M%S).log"
BINARY="./target/release/mesh-generator"
TIMEOUT_DURATION=10s
TOTAL_TESTS=15

# Colors for progress display
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Progress tracking
current_test=0

# Function to log with timestamp
log_with_timestamp() {
    local message="$1"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "[$timestamp] $message" | tee -a "$LOG_FILE"
}

# Function to show progress
show_progress() {
    current_test=$((current_test + 1))
    local percentage=$((current_test * 100 / TOTAL_TESTS))
    printf "\r${BLUE}Progress: [%-20s] %d%% (Test %d/%d)${NC}" \
        $(printf '#%.0s' $(seq 1 $((percentage/5)))) \
        $percentage $current_test $TOTAL_TESTS
    echo
}

# Function to run test with timeout and capture results
run_test() {
    local test_name="$1"
    local test_input="$2"
    local test_mode="$3"
    local expected_behavior="$4"
    
    show_progress
    echo -e "${YELLOW}Running: $test_name${NC}"
    log_with_timestamp "=== TEST: $test_name ==="
    log_with_timestamp "Input: $test_input"
    log_with_timestamp "Mode: $test_mode"
    log_with_timestamp "Expected: $expected_behavior"
    
    # Run the test and capture output
    local start_time=$(date +%s.%3N)
    local output
    local exit_code
    
    output=$(echo "$test_input" | timeout $TIMEOUT_DURATION $BINARY $test_mode 2>&1)
    exit_code=$?
    
    local end_time=$(date +%s.%3N)
    local duration=$(echo "$end_time - $start_time" | bc -l 2>/dev/null || echo "N/A")
    
    # Log results
    log_with_timestamp "Exit Code: $exit_code"
    log_with_timestamp "Duration: ${duration}s"
    
    if [ $exit_code -eq 124 ]; then
        echo -e "${RED}TIMEOUT${NC} (>${TIMEOUT_DURATION})"
        log_with_timestamp "Result: TIMEOUT"
    elif [ $exit_code -eq 0 ]; then
        echo -e "${GREEN}SUCCESS${NC} (${duration}s)"
        log_with_timestamp "Result: SUCCESS"
        # Count elements in output for summary
        local vertex_count=$(echo "$output" | jq '.vertices | length' 2>/dev/null || echo "N/A")
        local triangle_count=$(echo "$output" | jq '.triangles | length' 2>/dev/null || echo "N/A")
        local quad_count=$(echo "$output" | jq '.quads | length // 0' 2>/dev/null || echo "N/A")
        log_with_timestamp "Vertices: $vertex_count, Triangles: $triangle_count, Quads: $quad_count"
    else
        echo -e "${RED}FAILED${NC} (exit code: $exit_code)"
        log_with_timestamp "Result: FAILED"
    fi
    
    log_with_timestamp "Output: $output"
    log_with_timestamp ""
    echo
}

# Main test execution
main() {
    echo -e "${BLUE}=== Mesh Generator Test Suite ===${NC}"
    echo "Log file: $LOG_FILE"
    echo "Timeout per test: $TIMEOUT_DURATION"
    echo "Total tests: $TOTAL_TESTS"
    echo
    
    log_with_timestamp "=== MESH GENERATOR TEST SUITE START ==="
    log_with_timestamp "Binary: $BINARY"
    log_with_timestamp "Timeout: $TIMEOUT_DURATION"
    log_with_timestamp "Total Tests: $TOTAL_TESTS"
    log_with_timestamp ""
    
    # Build the project first
    echo -e "${YELLOW}Building Rust binary...${NC}"
    log_with_timestamp "=== BUILD PHASE ==="
    if cargo build --release >> "$LOG_FILE" 2>&1; then
        echo -e "${GREEN}Build successful!${NC}"
        log_with_timestamp "Build: SUCCESS"
    else
        echo -e "${RED}Build failed!${NC}"
        log_with_timestamp "Build: FAILED"
        exit 1
    fi
    log_with_timestamp ""
    echo
    
    # Test 1: Basic triangulation (minimal)
    run_test "Basic Triangle (3 points)" \
        '{"geometry":{"points":[{"x":0.0,"y":0.0},{"x":100.0,"y":0.0},{"x":50.0,"y":100.0}],"name":"triangle"},"max_area":null,"min_angle":null}' \
        "json-stdin" \
        "Single triangle output"
    
    # Test 2: Square with Delaunay (default algorithm)
    run_test "Square - Delaunay Algorithm" \
        '{"geometry":{"points":[{"x":0.0,"y":0.0},{"x":100.0,"y":0.0},{"x":100.0,"y":100.0},{"x":0.0,"y":100.0}],"name":"square"},"max_area":200.0,"min_angle":20.0,"algorithm":"delaunay"}' \
        "json-stdin" \
        "Multiple triangles with area constraint"
    
    # Test 3: Square with Paving (quad-dominant)
    run_test "Square - Paving Algorithm" \
        '{"geometry":{"points":[{"x":0.0,"y":0.0},{"x":100.0,"y":0.0},{"x":100.0,"y":100.0},{"x":0.0,"y":100.0}],"name":"square"},"max_area":200.0,"min_angle":20.0,"algorithm":"paving"}' \
        "json-stdin" \
        "Structured quad mesh"
    
    # Test 4: Larger geometry with fine mesh
    run_test "Large Rectangle - Fine Mesh" \
        '{"geometry":{"points":[{"x":0.0,"y":0.0},{"x":200.0,"y":0.0},{"x":200.0,"y":150.0},{"x":0.0,"y":150.0}],"name":"rectangle"},"max_area":50.0,"min_angle":25.0,"algorithm":"delaunay"}' \
        "json-stdin" \
        "Dense triangular mesh"
    
    # Test 5: Complex polygon (hexagon)
    run_test "Hexagon - Delaunay" \
        '{"geometry":{"points":[{"x":50.0,"y":0.0},{"x":100.0,"y":25.0},{"x":100.0,"y":75.0},{"x":50.0,"y":100.0},{"x":0.0,"y":75.0},{"x":0.0,"y":25.0}],"name":"hexagon"},"max_area":100.0,"min_angle":20.0,"algorithm":"delaunay"}' \
        "json-stdin" \
        "Triangulated hexagon"
    
    # Test 6: CSV export test
    run_test "CSV Export - Triangle" \
        '{"geometry":{"points":[{"x":0.0,"y":0.0},{"x":100.0,"y":0.0},{"x":50.0,"y":100.0}],"name":"csv_test"},"max_area":50.0,"min_angle":20.0,"algorithm":"delaunay"}' \
        "csv-stdin" \
        "CSV formatted output"
    
    # Test 7: Very coarse mesh
    run_test "Coarse Mesh - Large Area" \
        '{"geometry":{"points":[{"x":0.0,"y":0.0},{"x":200.0,"y":0.0},{"x":200.0,"y":200.0},{"x":0.0,"y":200.0}],"name":"coarse"},"max_area":2000.0,"min_angle":15.0,"algorithm":"delaunay"}' \
        "json-stdin" \
        "Few large triangles"
    
    # Test 8: High angle constraint
    run_test "High Angle Constraint" \
        '{"geometry":{"points":[{"x":0.0,"y":0.0},{"x":100.0,"y":0.0},{"x":100.0,"y":100.0},{"x":0.0,"y":100.0}],"name":"high_angle"},"max_area":300.0,"min_angle":35.0,"algorithm":"delaunay"}' \
        "json-stdin" \
        "Well-shaped triangles"
    
    # Test 9: No constraints (basic Delaunay)
    run_test "No Constraints - Pure Delaunay" \
        '{"geometry":{"points":[{"x":0.0,"y":0.0},{"x":120.0,"y":0.0},{"x":120.0,"y":80.0},{"x":0.0,"y":80.0}],"name":"pure"},"max_area":null,"min_angle":null,"algorithm":"delaunay"}' \
        "json-stdin" \
        "Basic Delaunay triangulation"
    
    # Test 10: Pentagon with paving
    run_test "Pentagon - Paving Algorithm" \
        '{"geometry":{"points":[{"x":50.0,"y":0.0},{"x":95.0,"y":35.0},{"x":80.0,"y":90.0},{"x":20.0,"y":90.0},{"x":5.0,"y":35.0}],"name":"pentagon"},"max_area":150.0,"min_angle":20.0,"algorithm":"paving"}' \
        "json-stdin" \
        "Paving mesh of pentagon"
    
    # Test 11: Stress test - very fine mesh
    run_test "Stress Test - Very Fine Mesh" \
        '{"geometry":{"points":[{"x":0.0,"y":0.0},{"x":100.0,"y":0.0},{"x":100.0,"y":100.0},{"x":0.0,"y":100.0}],"name":"stress"},"max_area":10.0,"min_angle":30.0,"algorithm":"delaunay"}' \
        "json-stdin" \
        "Many small high-quality triangles"
    
    # Test 12: Long thin rectangle
    run_test "Long Rectangle - Aspect Ratio Test" \
        '{"geometry":{"points":[{"x":0.0,"y":0.0},{"x":300.0,"y":0.0},{"x":300.0,"y":50.0},{"x":0.0,"y":50.0}],"name":"thin"},"max_area":100.0,"min_angle":20.0,"algorithm":"delaunay"}' \
        "json-stdin" \
        "Triangulation of thin geometry"
    
    # Test 13: Different algorithm comparison - same geometry
    run_test "Algorithm Comparison - Delaunay vs Previous" \
        '{"geometry":{"points":[{"x":0.0,"y":0.0},{"x":80.0,"y":0.0},{"x":80.0,"y":80.0},{"x":0.0,"y":80.0}],"name":"compare"},"max_area":200.0,"min_angle":25.0,"algorithm":"delaunay"}' \
        "json-stdin" \
        "Baseline for algorithm comparison"
    
    # Test 14: Edge case - very small geometry
    run_test "Small Geometry Test" \
        '{"geometry":{"points":[{"x":0.0,"y":0.0},{"x":10.0,"y":0.0},{"x":10.0,"y":10.0},{"x":0.0,"y":10.0}],"name":"small"},"max_area":5.0,"min_angle":20.0,"algorithm":"delaunay"}' \
        "json-stdin" \
        "Small scale meshing"
    
    # Test 15: Large coordinate values
    run_test "Large Coordinates Test" \
        '{"geometry":{"points":[{"x":1000.0,"y":1000.0},{"x":1200.0,"y":1000.0},{"x":1200.0,"y":1200.0},{"x":1000.0,"y":1200.0}],"name":"large_coords"},"max_area":500.0,"min_angle":20.0,"algorithm":"delaunay"}' \
        "json-stdin" \
        "Large coordinate handling"
    
    # Test summary
    echo
    echo -e "${BLUE}=== Test Suite Complete ===${NC}"
    echo "Results logged to: $LOG_FILE"
    
    log_with_timestamp "=== TEST SUITE SUMMARY ==="
    log_with_timestamp "All tests completed."
    log_with_timestamp "Check individual test results above for detailed analysis."
    
    # Generate summary statistics
    local success_count=$(grep -c "Result: SUCCESS" "$LOG_FILE" || echo "0")
    local timeout_count=$(grep -c "Result: TIMEOUT" "$LOG_FILE" || echo "0")
    local failed_count=$(grep -c "Result: FAILED" "$LOG_FILE" || echo "0")
    
    echo
    echo -e "${GREEN}Successful tests: $success_count${NC}"
    echo -e "${YELLOW}Timed out tests: $timeout_count${NC}"
    echo -e "${RED}Failed tests: $failed_count${NC}"
    
    log_with_timestamp "SUCCESS: $success_count, TIMEOUT: $timeout_count, FAILED: $failed_count"
    log_with_timestamp "=== END OF TEST SUITE ==="
    
    if [ $timeout_count -gt 0 ]; then
        echo
        echo -e "${YELLOW}Note: Timeouts suggest mesh refinement algorithms may need optimization${NC}"
    fi
    
    if [ $failed_count -gt 0 ]; then
        echo
        echo -e "${RED}Note: Some tests failed - check log for details${NC}"
    fi
}

# Check dependencies
check_dependencies() {
    command -v jq >/dev/null 2>&1 || {
        echo -e "${YELLOW}Warning: jq not found. Install with: sudo apt-get install jq${NC}"
        echo "Test will continue but JSON parsing in logs will be limited."
    }
    
    command -v bc >/dev/null 2>&1 || {
        echo -e "${YELLOW}Warning: bc not found. Install with: sudo apt-get install bc${NC}"
        echo "Test will continue but timing precision will be reduced."
    }
}

# Script entry point
if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    echo "Mesh Generator Test Suite"
    echo "Usage: $0 [options]"
    echo
    echo "This script runs comprehensive tests of the mesh generator with various:"
    echo "- Algorithms (Delaunay, Paving)"
    echo "- Geometries (triangle, square, hexagon, etc.)"
    echo "- Parameters (area constraints, angle constraints)"
    echo "- Output formats (JSON, CSV)"
    echo
    echo "Results are logged to a timestamped file with progress display."
    echo "Tests include timeout handling and performance measurement."
    exit 0
fi

# Run dependency check and main test suite
check_dependencies
main