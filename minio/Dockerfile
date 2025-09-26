FROM minio/minio:latest

# Install mc (MinIO Client)
RUN microdnf update && microdnf install curl -y && microdnf clean all

# Download and install mc client
RUN curl https://dl.min.io/client/mc/release/linux-amd64/mc -o /usr/local/bin/mc && \
    chmod +x /usr/local/bin/mc

# Create data directory
RUN mkdir -p /data

# Set environment variables
ENV MINIO_ROOT_USER=minio
ENV MINIO_ROOT_PASSWORD=minio123456
ENV MINIO_BROWSER=on

# Expose ports
EXPOSE 9000 9001

# Start MinIO server
CMD ["minio", "server", "/data", "--console-address", ":9001"]