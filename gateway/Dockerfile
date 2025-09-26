FROM kong:3.4.0-alpine

# Install custom plugins
COPY plugins/jwt-fruit.lua /usr/local/share/lua/5.1/kong/plugins/jwt-fruit/

# Copy configuration
COPY kong.yml /opt/kong/

# Expose ports
EXPOSE 8000 8001 8002 8003 8004 8443 8444 8445

# Start Kong
CMD ["kong", "docker-start", "--declarative-config", "/opt/kong/kong.yml"]