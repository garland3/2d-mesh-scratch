#!/bin/bash

# 2D Geometry & FEA Mesh Generator Setup Script
# Usage: ./setup.sh [--rust]

set -e

echo "🔧 Setting up 2D Geometry & FEA Mesh Generator..."

# Check if Python is available
if ! command -v python3 &> /dev/null; then
    echo "❌ Python 3 is required but not installed. Please install Python 3.7+ first."
    exit 1
fi

# Check if pip is available
if ! command -v pip3 &> /dev/null; then
    echo "❌ pip3 is required but not installed. Please install pip3 first."
    exit 1
fi

# Function to install Rust
install_rust() {
    echo "🦀 Installing Rust toolchain..."
    
    if command -v rustc &> /dev/null; then
        echo "✅ Rust is already installed ($(rustc --version))"
        return 0
    fi
    
    # Install Rust using rustup
    echo "📥 Downloading and installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    
    # Source the environment
    source ~/.cargo/env
    
    echo "✅ Rust installed successfully ($(rustc --version))"
}

# Function to build Rust binary
build_rust_binary() {
    echo "🔨 Building Rust binary..."
    
    if [ ! -f "target/release/mesh-generator" ]; then
        echo "📦 Compiling mesh-generator binary..."
        cargo build --release
        echo "✅ Rust binary built successfully"
    else
        echo "✅ Rust binary already exists"
    fi
}

# Function to setup Python environment
setup_python_env() {
    echo "🐍 Setting up Python environment..."
    
    # Create virtual environment if it doesn't exist
    if [ ! -d "venv" ]; then
        echo "📦 Creating virtual environment..."
        python3 -m venv venv
    fi
    
    # Activate virtual environment
    source venv/bin/activate
    
    # Upgrade pip
    pip install --upgrade pip
    
    # Install Python dependencies
    echo "📥 Installing Python dependencies..."
    pip install -r requirements.txt
    
    echo "✅ Python environment setup complete"
}

# Function to create necessary directories
create_directories() {
    echo "📁 Creating necessary directories..."
    
    mkdir -p static
    mkdir -p imgs
    
    echo "✅ Directories created"
}

# Function to check binary permissions
check_binary_permissions() {
    if [ -f "target/release/mesh-generator" ]; then
        chmod +x target/release/mesh-generator
        echo "✅ Binary permissions set"
    fi
}

# Function to run tests
run_tests() {
    echo "🧪 Running basic tests..."
    
    # Test if binary exists and runs
    if [ -f "target/release/mesh-generator" ]; then
        echo "Testing Rust binary..."
        if ./target/release/mesh-generator test > /dev/null 2>&1; then
            echo "✅ Rust binary test passed"
        else
            echo "⚠️  Rust binary test failed, but continuing..."
        fi
    fi
    
    # Test Python imports
    if source venv/bin/activate 2>/dev/null && python3 -c "import fastapi, uvicorn, pydantic" 2>/dev/null; then
        echo "✅ Python dependencies test passed"
    else
        echo "⚠️  Python dependencies test failed, but continuing..."
    fi
}

# Main setup function
main() {
    local install_rust_flag=false
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --rust)
                install_rust_flag=true
                shift
                ;;
            --help|-h)
                echo "Usage: $0 [--rust]"
                echo ""
                echo "Options:"
                echo "  --rust    Install Rust toolchain and build from source"
                echo "  --help    Show this help message"
                exit 0
                ;;
            *)
                echo "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done
    
    echo "🚀 Starting setup process..."
    
    # Create directories
    create_directories
    
    # Setup Python environment
    setup_python_env
    
    # Handle Rust installation and compilation
    if [ "$install_rust_flag" = true ]; then
        install_rust
        build_rust_binary
    else
        # Check if binary exists
        if [ ! -f "target/release/mesh-generator" ]; then
            echo "❌ Rust binary not found at target/release/mesh-generator"
            echo "💡 Either:"
            echo "   1. Run './setup.sh --rust' to install Rust and build from source"
            echo "   2. Or ensure the pre-compiled binary is in the correct location"
            exit 1
        else
            echo "✅ Using existing Rust binary"
        fi
    fi
    
    # Set binary permissions
    check_binary_permissions
    
    # Run tests
    run_tests
    
    echo ""
    echo "🎉 Setup complete!"
    echo ""
    echo "Next steps:"
    echo "1. Activate the virtual environment: source venv/bin/activate"
    echo "2. Start the server: python main.py"
    echo "3. Open http://localhost:8000 in your browser"
    echo ""
    echo "For development:"
    echo "- View logs: tail -f log"
    echo "- Test CLI: ./target/release/mesh-generator test"
    echo "- Fast test: ./fast_test.sh"
    echo ""
    echo "📚 Check README.md for detailed usage instructions"
}

# Run main function
main "$@"