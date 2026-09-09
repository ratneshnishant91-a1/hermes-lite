# Hermes-Lite v1.5 - Production Container

FROM python:3.11-slim

WORKDIR /app

# Install dependencies
RUN pip install --no-cache-dir PyYAML docker fastapi uvicorn python-multipart psutil

# Copy application
COPY . /app

# Create workspace
RUN mkdir -p /app/workspace/files

# Expose dashboard port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD python -c "from health import HealthChecker; from config import Config; h = HealthChecker(Config.load()); r = h.check_readiness(); exit(0 if r['all_healthy'] else 1)"

# Run dashboard
CMD ["python", "dashboard.py"]
