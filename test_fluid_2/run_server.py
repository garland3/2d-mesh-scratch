#!/usr/bin/env python3
import subprocess
import os
import sys
from pathlib import Path

def build_wasm():
    """Build the Rust WASM module"""
    print("Building Rust WASM module...")
    try:
        result = subprocess.run(["wasm-pack", "build", "--target", "web", "--out-dir", "pkg"], 
                              cwd=Path(__file__).parent, check=True)
        print("✓ WASM build completed successfully!")
        return True
    except subprocess.CalledProcessError as e:
        print(f"✗ WASM build failed: {e}")
        return False
    except FileNotFoundError:
        print("✗ wasm-pack not found. Please install it with: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh")
        return False

def main():
    # Change to the script directory
    os.chdir(Path(__file__).parent)
    
    # Build WASM first
    if not build_wasm():
        sys.exit(1)
    
    # Check if FastAPI is installed
    try:
        import fastapi
        import uvicorn
    except ImportError:
        print("Installing FastAPI and uvicorn...")
        subprocess.run([sys.executable, "-m", "pip", "install", "fastapi", "uvicorn[standard]"], check=True)
        import fastapi
        import uvicorn
    
    from fastapi import FastAPI
    from fastapi.staticfiles import StaticFiles
    from fastapi.responses import FileResponse
    
    app = FastAPI(title="Fluid Simulator", description="Rust WASM Fluid Simulation")
    
    # Serve static files
    app.mount("/pkg", StaticFiles(directory="pkg"), name="pkg")
    app.mount("/js", StaticFiles(directory="js"), name="js")
    
    @app.get("/")
    async def root():
        return FileResponse("example.html")
    
    print("\n🚀 Starting fluid simulator server...")
    print("📍 Open your browser to: http://localhost:8000")
    print("🛑 Press Ctrl+C to stop the server\n")
    
    uvicorn.run(app, host="0.0.0.0", port=8000, log_level="info")

if __name__ == "__main__":
    main()