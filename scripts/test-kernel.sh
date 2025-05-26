#!/bin/bash

# Simple script to test our kernel without Jupyter
echo "🧪 Testing wabznasm Jupyter kernel..."

# Create a test connection file
cat > /tmp/test-connection.json << EOF
{
  "shell_port": 54321,
  "iopub_port": 54322,
  "stdin_port": 54323,
  "control_port": 54324,
  "hb_port": 54325,
  "ip": "127.0.0.1",
  "key": "test-key-123",
  "transport": "tcp",
  "signature_scheme": "hmac-sha256",
  "kernel_name": "wabznasm"
}
EOF

echo "📄 Created test connection file at /tmp/test-connection.json"
echo "🚀 Starting kernel (Ctrl+C to stop)..."
echo ""

# Build and run our kernel
cargo run -- jupyter start /tmp/test-connection.json
