# Multi-stage Dockerfile for App4.Dog Game
# Optimized for pre-built WASM artifacts from CI

FROM nginx:alpine

# Install Node.js for any runtime needs
RUN apk add --no-cache nodejs npm

# Copy pre-built application files
COPY dist/spa /usr/share/nginx/html

# Copy WASM game engine files
COPY public/game-engine /usr/share/nginx/html/game-engine

# Create nginx configuration for SPA
RUN cat > /etc/nginx/conf.d/default.conf << 'EOF'
server {
    listen 80;
    server_name localhost;
    root /usr/share/nginx/html;
    index index.html;

    # Enable gzip compression
    gzip on;
    gzip_vary on;
    gzip_min_length 1024;
    gzip_types
        text/plain
        text/css
        text/xml
        text/javascript
        application/javascript
        application/xml+rss
        application/json
        application/wasm;

    # SPA routing - serve index.html for all routes
    location / {
        try_files $uri $uri/ /index.html;
        
        # Add headers for WASM and modern web features
        add_header Cross-Origin-Embedder-Policy "require-corp" always;
        add_header Cross-Origin-Opener-Policy "same-origin" always;
        
        # Cache static assets
        location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|wasm)$ {
            expires 1y;
            add_header Cache-Control "public, immutable";
        }
    }

    # Serve WASM files with correct MIME type
    location ~* \.wasm$ {
        add_header Content-Type application/wasm;
        add_header Cross-Origin-Embedder-Policy "require-corp" always;
        add_header Cross-Origin-Opener-Policy "same-origin" always;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    # Game engine API endpoints (if any)
    location /api/ {
        # Proxy to backend if needed
        # proxy_pass http://backend:8080;
    }
    
    # Health check endpoint
    location /health {
        access_log off;
        return 200 "healthy\n";
        add_header Content-Type text/plain;
    }
}
EOF

# Expose port
EXPOSE 80

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost/health || exit 1

# Start nginx
CMD ["nginx", "-g", "daemon off;"]