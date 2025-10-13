from datetime import datetime
from typing import Dict, List

from pydantic import BaseModel, ConfigDict, field_serializer


class DetectionResult(BaseModel):
    class_name: str
    confidence: float
    bbox: List[float]
    quality_grade: str


class InferenceResponse(BaseModel):
    model_config = ConfigDict(
        # Use custom serializers instead of deprecated json_encoders
        ser_json_timedelta='iso8601',
    )

    upload_id: str
    status: str
    total_fruits: int
    timestamp: datetime
    results: List[DetectionResult]
    metadata: Dict[str, str]

    @field_serializer('timestamp')
    def serialize_timestamp(self, timestamp: datetime) -> str:
        """Serialize timestamp to ISO format."""
        return timestamp.isoformat()
