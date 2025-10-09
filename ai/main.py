import asyncio
import json
import logging
import os
from datetime import datetime
from typing import Dict, List, Optional

import torch
import torch.nn as nn
import cv2
import numpy as np
from torchvision import models
from kafka import KafkaConsumer, KafkaProducer
from kafka.errors import KafkaError
import boto3
from botocore.exceptions import ClientError

from utils.preprocess import preprocess_image, DEFAULT_TRANSFORM
from utils.postprocess import calculate_quality_grade

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

class FruitDetectionService:
    def __init__(self):
        self.model = None
        self.device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
        self.transform = DEFAULT_TRANSFORM
        self.s3_client = None
        self.kafka_consumer = None
        self.kafka_producer = None
        self.labels = self._load_labels()
        self.num_classes = len(self.labels)
        self.top_k = min(3, self.num_classes) if self.num_classes else 0

        self._init_model()
        self._init_s3_client()
        self._init_kafka()

    def _load_labels(self) -> Dict[int, str]:
        """Load class labels from JSON file."""
        try:
            with open('model/labels.json', 'r', encoding='utf-8') as f:
                raw_labels = json.load(f)

            labels = {int(k): v for k, v in raw_labels.items()}
            logger.info(f"Loaded {len(labels)} class labels")
            return labels
        except FileNotFoundError:
            logger.warning("Labels file not found, using default fruit classes")
            return {
                0: "apple", 1: "banana", 2: "orange", 3: "grape",
                4: "strawberry", 5: "pineapple", 6: "watermelon", 7: "peach"
            }

    def _init_model(self):
        """Initialize DenseNet model."""
        try:
            logger.info("Initializing DenseNet model for fruit classification")
            base_model = models.densenet121(weights=models.DenseNet121_Weights.IMAGENET1K_V1)

            # Replace final classifier to match dataset classes
            num_features = base_model.classifier.in_features
            if self.num_classes == 0:
                raise ValueError("No class labels available for model initialization")

            base_model.classifier = nn.Linear(num_features, self.num_classes)

            model_path = 'model/fruit_densenet_weights.pth'
            if os.path.exists(model_path):
                state_dict = torch.load(model_path, map_location=self.device)
                base_model.load_state_dict(state_dict)
                logger.info(f"Loaded DenseNet weights from {model_path}")
            else:
                logger.warning(
                    "Custom DenseNet weights not found. Using ImageNet backbone with randomly "
                    "initialized classifier; predictions may be unreliable."
                )

            base_model.to(self.device)
            base_model.eval()

            self.model = base_model
            logger.info(f"Model loaded on device: {self.device}")

        except Exception as e:
            logger.error(f"Failed to initialize model: {e}")
            raise

    def _init_s3_client(self):
        """Initialize S3/MinIO client."""
        try:
            self.s3_client = boto3.client(
                's3',
                endpoint_url='http://minio:9000',
                aws_access_key_id='minio',
                aws_secret_access_key='minio123456',
                region_name='us-east-1'
            )
            logger.info("S3 client initialized")
        except Exception as e:
            logger.error(f"Failed to initialize S3 client: {e}")
            raise

    def _init_kafka(self):
        """Initialize Kafka consumer and producer."""
        try:
            self.kafka_consumer = KafkaConsumer(
                'image-events',
                bootstrap_servers=['kafka:9092'],
                auto_offset_reset='latest',
                enable_auto_commit=True,
                group_id='ai-inference-group',
                value_deserializer=lambda x: json.loads(x.decode('utf-8'))
            )

            self.kafka_producer = KafkaProducer(
                bootstrap_servers=['kafka:9092'],
                value_serializer=lambda x: json.dumps(x).encode('utf-8')
            )

            logger.info("Kafka consumer and producer initialized")
        except Exception as e:
            logger.error(f"Failed to initialize Kafka: {e}")
            raise

    def download_image_from_s3(self, object_key: str) -> Optional[np.ndarray]:
        """Download image from S3/MinIO storage."""
        try:
            response = self.s3_client.get_object(Bucket='fruit-images', Key=object_key)
            image_data = response['Body'].read()

            # Convert bytes to numpy array
            nparr = np.frombuffer(image_data, np.uint8)
            image = cv2.imdecode(nparr, cv2.IMREAD_COLOR)

            if image is None:
                logger.error(f"Failed to decode image: {object_key}")
                return None

            logger.info(f"Downloaded image: {object_key}, shape: {image.shape}")
            return image

        except ClientError as e:
            logger.error(f"Failed to download image {object_key}: {e}")
            return None
        except Exception as e:
            logger.error(f"Unexpected error downloading image {object_key}: {e}")
            return None

    def detect_fruits(self, image: np.ndarray) -> List[Dict]:
        """Perform fruit classification on image."""
        try:
            if self.model is None or self.top_k == 0:
                logger.error("Model is not initialized or labels are missing")
                return []

            # Preprocess image into tensor and move to device
            processed_tensor = preprocess_image(image, self.transform).unsqueeze(0).to(self.device)

            with torch.no_grad():
                logits = self.model(processed_tensor)
                probabilities = torch.softmax(logits, dim=1)
                top_probs, top_indices = probabilities.topk(self.top_k, dim=1)

            detections = []
            height, width = image.shape[:2]

            for probability, class_index in zip(top_probs[0], top_indices[0]):
                confidence = float(probability.item())
                class_id = int(class_index.item())
                class_name = self.labels.get(class_id, f"unknown_{class_id}")

                # Use full image as region since DenseNet provides classification only
                quality_grade = calculate_quality_grade(
                    image,
                    class_name,
                    confidence
                )

                detections.append({
                    'class_name': class_name,
                    'confidence': confidence,
                    'bbox': [0.0, 0.0, float(width), float(height)],
                    'quality_grade': quality_grade
                })

            logger.info(f"Generated top-{len(detections)} predictions")
            return detections

        except Exception as e:
            logger.error(f"Error during fruit detection: {e}")
            return []

    def send_result_to_kafka(self, upload_id: str, detections: List[Dict], status: str = "completed"):
        """Send detection results to Kafka."""
        try:
            result_message = {
                'event_type': 'inference.completed',
                'upload_id': upload_id,
                'status': status,
                'results': detections,
                'total_fruits': len(detections),
                'timestamp': datetime.utcnow().isoformat(),
                'metadata': {
                    'model_version': 'densenet121-fruit-v1',
                    'confidence_threshold': None,
                    'top_k': self.top_k
                }
            }

            future = self.kafka_producer.send('inference-results', value=result_message)
            future.get(timeout=10)  # Wait for message to be sent

            logger.info(f"Sent results to Kafka for upload_id: {upload_id}")

        except KafkaError as e:
            logger.error(f"Failed to send results to Kafka: {e}")
        except Exception as e:
            logger.error(f"Unexpected error sending to Kafka: {e}")

    async def process_message(self, message):
        """Process a single Kafka message."""
        try:
            logger.info(f"Processing message: {message}")

            upload_id = message.get('upload_id')
            object_key = message.get('object_key')

            if not upload_id or not object_key:
                logger.error("Missing required fields in message")
                return

            # Download image from S3
            image = self.download_image_from_s3(object_key)
            if image is None:
                self.send_result_to_kafka(upload_id, [], "failed")
                return

            # Perform detection
            detections = self.detect_fruits(image)

            # Send results
            self.send_result_to_kafka(upload_id, detections, "completed")

        except Exception as e:
            logger.error(f"Error processing message: {e}")
            # Send failure notification
            if 'upload_id' in locals():
                self.send_result_to_kafka(upload_id, [], "failed")

    def run(self):
        """Run the main processing loop."""
        logger.info("Starting AI inference service...")

        try:
            for message in self.kafka_consumer:
                if message.value.get('event_type') == 'image.stored':
                    asyncio.run(self.process_message(message.value))

        except KeyboardInterrupt:
            logger.info("Shutting down service...")
        except Exception as e:
            logger.error(f"Error in main loop: {e}")
        finally:
            if self.kafka_consumer:
                self.kafka_consumer.close()
            if self.kafka_producer:
                self.kafka_producer.close()

def main():
    service = FruitDetectionService()
    service.run()

if __name__ == "__main__":
    main()
