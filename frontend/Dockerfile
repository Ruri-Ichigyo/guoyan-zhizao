# Multi-stage build for Vue.js frontend

# Build stage
FROM node:18-alpine as build-stage

# Set working directory
WORKDIR /app

# Copy package files first to leverage Docker layer caching
COPY package.json package-lock.json ./

# Install dependencies
RUN npm ci

# Copy source code
COPY . .

# Build arguments for environment configuration
ARG NODE_ENV=production
ARG VITE_API_BASE_URL=http://localhost:8080/api/v1
ARG VITE_APP_NAME="果蔬智造核心平台"
ARG VITE_APP_VERSION=1.0.0

# Set environment variables
ENV NODE_ENV=$NODE_ENV
ENV VITE_API_BASE_URL=$VITE_API_BASE_URL
ENV VITE_APP_NAME=$VITE_APP_NAME
ENV VITE_APP_VERSION=$VITE_APP_VERSION

# Build the application
RUN npm run build

# Production stage
FROM nginx:1.25-alpine as production-stage

# Install dumb-init for better signal handling
RUN apk add --no-cache dumb-init

# Create a non-root user
RUN addgroup -g 1001 -S nodejs && \
    adduser -S nextjs -u 1001

# Remove default nginx website
RUN rm -rf /usr/share/nginx/html/*

# Copy built application from build stage
COPY --from=build-stage /app/dist /usr/share/nginx/html

# Copy custom nginx main configuration for non-root user
COPY <<'EOF' /etc/nginx/nginx.conf
user nextjs;
worker_processes auto;
error_log /var/log/nginx/error.log warn;
pid /var/run/nginx/nginx.pid;

events {
    worker_connections 1024;
}

http {
    include /etc/nginx/mime.types;
    default_type application/octet-stream;

    log_format main '$remote_addr - $remote_user [$time_local] "$request" '
                    '$status $body_bytes_sent "$http_referer" '
                    '"$http_user_agent" "$http_x_forwarded_for"';

    access_log /var/log/nginx/access.log main;

    sendfile on;
    tcp_nopush on;
    tcp_nodelay on;
    keepalive_timeout 65;
    types_hash_max_size 2048;

    include /etc/nginx/conf.d/*.conf;
}
EOF

# Copy custom nginx site configuration
COPY <<'EOF' /etc/nginx/conf.d/default.conf
server {
    listen 80;
    server_name localhost;
    root /usr/share/nginx/html;
    index index.html index.htm;

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "no-referrer-when-downgrade" always;
    add_header Content-Security-Policy "default-src 'self' http: https: ws: wss: data: blob: 'unsafe-inline' 'unsafe-eval'" always;

    # Gzip compression
    gzip on;
    gzip_vary on;
    gzip_min_length 1024;
    gzip_types text/plain text/css text/xml text/javascript application/javascript application/xml+rss application/json;

    # Cache static assets
    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2|ttf|eot)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
        try_files $uri =404;
    }

    # Handle Vue.js routing - always serve index.html for non-existent files
    location / {
        try_files $uri $uri/ /index.html;
    }

    # API proxy (optional - for development/testing)
    location /api/v1/ {
        proxy_pass http://backend:8080/api/v1/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_connect_timeout 30s;
        proxy_send_timeout 30s;
        proxy_read_timeout 30s;
    }

    # Health check endpoint
    location /health {
        access_log off;
        return 200 "healthy\n";
        add_header Content-Type text/plain;
    }

    # Block access to sensitive files
    location ~ /\. {
        deny all;
        access_log off;
        log_not_found off;
    }

    location ~* \.(htaccess|htpasswd|ini|phps|fla|psd|log|sh)$ {
        deny all;
        access_log off;
        log_not_found off;
    }

    # Error pages
    error_page 404 /index.html;
    error_page 500 502 503 504 /index.html;
}
EOF

# Create startup script for environment variable substitution
COPY <<'EOF' /docker-entrypoint.sh
#!/bin/sh
set -e

# Substitute environment variables in JavaScript files at runtime
find /usr/share/nginx/html -name "*.js" -exec sed -i \
    -e "s|__VITE_API_BASE_URL__|${VITE_API_BASE_URL}|g" \
    -e "s|__VITE_APP_NAME__|${VITE_APP_NAME}|g" \
    -e "s|__VITE_APP_VERSION__|${VITE_APP_VERSION}|g" \
    {} \;

# Start nginx
exec "$@"
EOF

# Make startup script executable
RUN chmod +x /docker-entrypoint.sh

# Prepare runtime directories and adjust permissions
RUN set -eux; \
    for dir in /var/cache/nginx /var/cache/nginx/client_temp /var/cache/nginx/proxy_temp /var/cache/nginx/fastcgi_temp /var/cache/nginx/uwsgi_temp /var/cache/nginx/scgi_temp /var/run/nginx /tmp/nginx; do \
        mkdir -p "${dir}"; \
    done; \
    chown -R nextjs:nodejs /var/cache/nginx /var/run/nginx /tmp/nginx; \
    chown -R nextjs:nodejs /usr/share/nginx/html; \
    chmod -R 755 /usr/share/nginx/html; \
    touch /var/run/nginx/nginx.pid && chown nextjs:nodejs /var/run/nginx/nginx.pid

# Switch to non-root user
USER nextjs

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost/health || exit 1

# Expose port
EXPOSE 80

# Set entrypoint and command
ENTRYPOINT ["/usr/bin/dumb-init", "--", "/docker-entrypoint.sh"]
CMD ["nginx", "-g", "daemon off;"]

# Labels for metadata
LABEL maintainer="guoyan-zhizao-team@example.com"
LABEL description="Frontend for 果蔬智能分拣系统 (Intelligent Fruit and Vegetable Sorting System)"
LABEL version="1.0.0"
LABEL org.opencontainers.image.source="https://github.com/your-org/guoyan-zhizao"
LABEL org.opencontainers.image.documentation="https://github.com/your-org/guoyan-zhizao/README.md"
