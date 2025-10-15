from typing import Dict, Optional

from fastapi import FastAPI, File, Form, HTTPException, UploadFile
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse

from app.config import config
from app.exceptions import FruitDetectionError, ImageProcessingError, InferenceError
from app.models import model_registry
from app.service import FruitDetectionService
from app.logging_config import get_logger

logger = get_logger(__name__)

# Initialize service with default model
service = FruitDetectionService()

app = FastAPI(
    title="Fruit Detection Service",
    description="AI inference service for fruit sorting",
    version=config.SERVICE_VERSION,
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


@app.get("/health")
async def health() -> Dict[str, str]:
    """Service health check endpoint."""
    return {"status": "ok"}


@app.get("/models")
async def list_models() -> Dict[str, list]:
    """List all available models."""
    models = model_registry.list_models()
    return {"models": models}


@app.get("/models/{model_name}")
async def get_model_info(model_name: str) -> Dict[str, str]:
    """Get information about a specific model."""
    try:
        model = model_registry.get_model(model_name)
        return {
            "name": model.name,
            "version": model.version,
            "num_classes": str(model.num_classes),
            "device": str(model.device)
        }
    except FruitDetectionError as exc:
        logger.error("Model loading error: %s", exc.message)
        raise HTTPException(status_code=404, detail=exc.message) from exc
    except Exception as exc:
        logger.error("Unexpected error getting model info: %s", exc)
        raise HTTPException(status_code=500, detail="Internal server error") from exc


@app.post("/infer")
async def infer(
    upload_id: str = Form(...),
    file: UploadFile = File(...),
    model_name: Optional[str] = Form(None)
) -> JSONResponse:
    """Perform inference on an uploaded image."""
    try:
        bytes_data = await file.read()
        logger.info(
            "Received inference request for %s with model %s (%s bytes)", 
            upload_id, 
            model_name or config.DEFAULT_MODEL_NAME,
            len(bytes_data)
        )
        
        result = service.infer(upload_id, bytes_data, model_name)

        response_dict = {
            "upload_id": str(result.upload_id),
            "status": result.status,
            "total_fruits": result.total_fruits,
            "timestamp": result.timestamp.isoformat(),
            "results": [
                {
                    "class_name": r.class_name,
                    "confidence": r.confidence,
                    "bbox": r.bbox,
                    "quality_grade": r.quality_grade,
                }
                for r in result.results
            ],
            "metadata": result.metadata,
        }
        return JSONResponse(content=response_dict)
    except ImageProcessingError as exc:
        logger.error("Image processing error: %s", exc.message)
        raise HTTPException(status_code=400, detail=exc.message) from exc
    except InferenceError as exc:
        logger.error("Inference error: %s", exc.message)
        raise HTTPException(status_code=500, detail=exc.message) from exc
    except ValueError as exc:
        logger.error("Validation error: %s", exc)
        raise HTTPException(status_code=400, detail=str(exc)) from exc
    except FruitDetectionError as exc:
        logger.error("Fruit detection service error: %s", exc.message)
        status_code = 400 if exc.error_code and "VALIDATION" in exc.error_code else 500
        raise HTTPException(status_code=status_code, detail=exc.message) from exc
    except Exception as exc:  # pylint: disable=broad-except
        logger.exception("Unexpected error during inference: %s", exc)
        raise HTTPException(status_code=500, detail="Internal server error during inference") from exc


@app.get("/metrics")
async def get_metrics():
    """Get service metrics and statistics."""
    # In a real implementation, you might track inference counts, latency, etc.
    return {
        "service_version": config.SERVICE_VERSION,
        "model_loaded": config.DEFAULT_MODEL_NAME,
        "available_models": model_registry.list_models(),
        "img_size": config.IMG_SIZE,
        "use_gpu": config.USE_GPU
    }
