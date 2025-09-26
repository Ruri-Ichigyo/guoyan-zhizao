import asyncio
import json
import logging
import os
from datetime import datetime
from typing import Dict, List, Optional

import torch
import cv2
import numpy as np
from ultralytics import YOLO
from kafka import KafkaConsumer, KafkaProducer
from kafka.errors import KafkaError
import boto3
from botocore.exceptions import ClientError

from utils.preprocess import preprocess_image
from utils.postprocess import postprocess_results, calculate_quality_grade

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

class FruitDetectionService:
    def __init__(self):
        self.model = None
        self.s3_client = None
        self.kafka_consumer = None
        self.kafka_producer = None
        self.labels = self._load_labels()

        self._init_model()
        self._init_s3_client()
        self._init_kafka()

    def _load_labels(self) -> Dict[int, str]:
        """Load class labels from JSON file."""
        try:
            with open('model/labels.json', 'r', encoding='utf-8') as f:
                labels = json.load(f)
            logger.info(f"Loaded {len(labels)} class labels")
            return labels
        except FileNotFoundError:
            logger.warning("Labels file not found, using default fruit classes")
            return {
                0: "apple", 1: "banana", 2: "orange", 3: "grape",
                4: "strawberry", 5: "pineapple", 6: "watermelon", 7: "peach"
            }

    def _init_model(self):
        """Initialize YOLO model."""
        try:
            model_path = 'model/yolov8n-fruit.pt'
            if os.path.exists(model_path):
                self.model = YOLO(model_path)
                logger.info(f"Loaded custom model from {model_path}")
            else:
                self.model = YOLO('yolov8n.pt')  # Use pretrained model as fallback
                logger.warning("Custom model not found, using pretrained YOLOv8n")

            # Set device
            device = 'cuda' if torch.cuda.is_available() else 'cpu'
            self.model.to(device)
            logger.info(f"Model loaded on device: {device}")

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
        """Perform fruit detection on image."""
        try:
            # Preprocess image
            processed_image = preprocess_image(image)

            # Run inference
            results = self.model(processed_image, verbose=False)

            # Process results
            detections = []
            for result in results:
                if result.boxes is not None:
                    boxes = result.boxes.xyxy.cpu().numpy()
                    confidences = result.boxes.conf.cpu().numpy()
                    classes = result.boxes.cls.cpu().numpy()

                    for i in range(len(boxes)):
                        x1, y1, x2, y2 = boxes[i]
                        confidence = float(confidences[i])
                        class_id = int(classes[i])

                        if confidence > 0.5:  # Confidence threshold
                            class_name = self.labels.get(class_id, f"unknown_{class_id}")

                            # Calculate quality grade based on various factors
                            quality_grade = calculate_quality_grade(
                                image[int(y1):int(y2), int(x1):int(x2)],
                                class_name,
                                confidence
                            )

                            detection = {
                                'class_name': class_name,
                                'confidence': confidence,
                                'bbox': [float(x1), float(y1), float(x2), float(y2)],
                                'quality_grade': quality_grade
                            }
                            detections.append(detection)

            logger.info(f"Detected {len(detections)} fruits")
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
                    'model_version': 'yolov8n-fruit-v1',
                    'confidence_threshold': 0.5
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