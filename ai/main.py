from app.api import app  # re-export FastAPI application for ASGI servers
from app.service import FruitDetectionService
from app.config import config

__all__ = ["app", "FruitDetectionService"]


if __name__ == "__main__":
    import uvicorn

    uvicorn.run(
        "app.api:app", 
        host=config.API_HOST, 
        port=config.API_PORT
    )
