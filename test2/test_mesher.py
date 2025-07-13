#!/usr/bin/env python3
"""
Simple test script to run a local HTTP server for testing the Rust mesher
"""
import http.server
import socketserver
import webbrowser
import time
import os

PORT = 8000

class Handler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=os.getcwd(), **kwargs)

def test_mesher():
    print("🚀 Starting Rust 2D Mesher Test Server...")
    print(f"📁 Serving from: {os.getcwd()}")
    
    with socketserver.TCPServer(("", PORT), Handler) as httpd:
        print(f"🌐 Server running at http://localhost:{PORT}")
        print("📋 Test checklist:")
        print("  1. Open browser and go to http://localhost:{PORT}")
        print("  2. Wait for 'WASM loaded. Ready to mesh!' message")
        print("  3. Click on canvas to draw a polygon (3+ points)")
        print("  4. Click 'Generate Mesh (Rust)' - should be very fast!")
        print("  5. Try refinement and smoothing operations")
        print("  6. Download the mesh JSON")
        print("\n⏱️  Performance should be significantly faster than JavaScript!")
        print("🔥 Rust + WASM should handle large meshes with ease\n")
        print("Press Ctrl+C to stop server")
        
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\n✅ Server stopped")

if __name__ == "__main__":
    test_mesher()