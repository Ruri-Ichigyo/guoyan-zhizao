FROM ghcr.io/astral-sh/uv:python3.12-bookworm-slim

# Configure proxy if needed
ARG HTTP_PROXY
ARG HTTPS_PROXY
ENV http_proxy=$HTTP_PROXY
ENV https_proxy=$HTTPS_PROXY

# Use Chinese mirror for faster downloads
ENV UV_INDEX_URL=https://mirrors.aliyun.com/pypi/simple/

# Install system dependencies for image processing
RUN apt-get update && apt-get install -y \
    libglib2.0-0 \
    libsm6 \
    libxext6 \
    libxrender-dev \
    libgomp1 \
    libgl1 \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy dependency files first to leverage Docker cache
COPY pyproject.toml uv.lock .uv.toml ./

# Install Python dependencies
# This step will be cached if pyproject.toml and uv.lock unchanged
RUN --mount=type=cache,target=/root/.cache/uv \
    uv sync --frozen

# Copy source code after dependencies are installed
COPY . .

# Create model directory
RUN mkdir -p /app/model

# Expose port
EXPOSE 8002

# Command to run the application
CMD ["/app/.venv/bin/uvicorn", "app.api:app", "--host", "0.0.0.0", "--port", "8002"]
