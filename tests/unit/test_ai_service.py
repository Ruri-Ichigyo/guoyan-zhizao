import pytest
import asyncio
import tempfile
import numpy as np
from unittest.mock import Mock, patch, AsyncMock
from PIL import Image
import io

# 添加项目路径
import sys
import os
sys.path.append(os.path.join(os.path.dirname(__file__), '../../ai'))

from main import FruitDetectionService
from utils.preprocess import preprocess_image, letterbox
from utils.postprocess import calculate_quality_grade, calculate_iou

class TestFruitDetectionService:
    """AI 推理服务单元测试"""

    @pytest.fixture
    def service(self):
        """创建测试服务实例"""
        with patch('main.YOLO'), \
             patch('main.boto3.client'), \
             patch('main.KafkaConsumer'), \
             patch('main.KafkaProducer'):
            return FruitDetectionService()

    @pytest.fixture
    def test_image(self):
        """创建测试图像"""
        # 创建 RGB 测试图像
        image = np.random.randint(0, 255, (480, 640, 3), dtype=np.uint8)
        return image

    @pytest.fixture
    def test_image_pil(self):
        """创建 PIL 格式测试图像"""
        image = Image.new('RGB', (640, 480), color='red')
        return image

    def test_load_labels(self, service):
        """测试标签加载"""
        assert isinstance(service.labels, dict)
        assert len(service.labels) > 0
        assert '苹果' in service.labels.values() or 'apple' in service.labels.values()

    def test_preprocess_image(self, test_image):
        """测试图像预处理"""
        processed = preprocess_image(test_image)

        assert processed.shape == (640, 640, 3)
        assert processed.dtype == np.float32
        assert 0 <= processed.min() <= processed.max() <= 1

    def test_letterbox_resize(self, test_image):
        """测试 letterbox 调整"""
        resized = letterbox(test_image, (640, 640))

        assert resized.shape == (640, 640, 3)
        assert resized.dtype == test_image.dtype

    def test_quality_grade_calculation(self):
        """测试质量分级计算"""
        # 创建高质量果蔬区域 (均匀颜色)
        high_quality_region = np.full((100, 100, 3), [200, 100, 50], dtype=np.uint8)
        grade_high = calculate_quality_grade(high_quality_region, "苹果", 0.95)
        assert grade_high in ['A', 'B', 'C']

        # 创建低质量果蔬区域 (噪声较多)
        low_quality_region = np.random.randint(0, 255, (100, 100, 3), dtype=np.uint8)
        grade_low = calculate_quality_grade(low_quality_region, "苹果", 0.6)
        assert grade_low in ['A', 'B', 'C']

        # 高质量应该获得更好的评级
        grade_order = {'A': 3, 'B': 2, 'C': 1}
        assert grade_order[grade_high] >= grade_order[grade_low]

    def test_iou_calculation(self):
        """测试 IoU 计算"""
        # 完全重叠的框
        box1 = [10, 10, 50, 50]
        box2 = [10, 10, 50, 50]
        iou = calculate_iou(box1, box2)
        assert abs(iou - 1.0) < 0.01

        # 完全不重叠的框
        box1 = [10, 10, 50, 50]
        box2 = [60, 60, 100, 100]
        iou = calculate_iou(box1, box2)
        assert abs(iou - 0.0) < 0.01

        # 部分重叠的框
        box1 = [10, 10, 50, 50]  # 40x40
        box2 = [30, 30, 70, 70]  # 40x40
        iou = calculate_iou(box1, box2)

        # 计算预期 IoU
        intersection_area = 20 * 20  # 400
        union_area = 40 * 40 + 40 * 40 - 400  # 2800
        expected_iou = intersection_area / union_area
        assert abs(iou - expected_iou) < 0.01

    def test_empty_fruit_region_quality(self):
        """测试空果蔬区域的质量评估"""
        empty_region = np.array([])
        grade = calculate_quality_grade(empty_region, "苹果", 0.8)
        assert grade == 'C'

    @patch('main.FruitDetectionService.download_image_from_s3')
    @patch('main.FruitDetectionService.detect_fruits')
    @patch('main.FruitDetectionService.send_result_to_kafka')
    @pytest.mark.asyncio
    async def test_process_message(self, mock_kafka, mock_detect, mock_download, service):
        """测试消息处理"""
        # 设置 Mock 返回值
        mock_download.return_value = np.random.randint(0, 255, (480, 640, 3), dtype=np.uint8)
        mock_detect.return_value = [
            {
                'class_name': '苹果',
                'confidence': 0.95,
                'bbox': [10, 10, 50, 50],
                'quality_grade': 'A'
            }
        ]

        # 测试消息
        test_message = {
            'upload_id': 'test-123',
            'object_key': 'test/image.jpg'
        }

        # 处理消息
        await service.process_message(test_message)

        # 验证调用
        mock_download.assert_called_once_with('test/image.jpg')
        mock_detect.assert_called_once()
        mock_kafka.assert_called_once()

    @patch('main.FruitDetectionService.download_image_from_s3')
    @patch('main.FruitDetectionService.send_result_to_kafka')
    @pytest.mark.asyncio
    async def test_process_message_download_failure(self, mock_kafka, mock_download, service):
        """测试图像下载失败的处理"""
        # 模拟下载失败
        mock_download.return_value = None

        test_message = {
            'upload_id': 'test-456',
            'object_key': 'test/missing.jpg'
        }

        await service.process_message(test_message)

        # 验证发送了失败消息
        mock_kafka.assert_called_once()
        args, kwargs = mock_kafka.call_args
        assert args[1] == []  # 空结果
        assert args[2] == "failed"  # 失败状态

    def test_detect_fruits_with_mock_model(self, service, test_image):
        """测试果蔬检测（使用模拟模型）"""
        # 模拟 YOLO 模型返回
        mock_result = Mock()
        mock_result.boxes = Mock()
        mock_result.boxes.xyxy.cpu().numpy.return_value = np.array([[10, 20, 100, 150]])
        mock_result.boxes.conf.cpu().numpy.return_value = np.array([0.95])
        mock_result.boxes.cls.cpu().numpy.return_value = np.array([0])

        with patch.object(service.model, '__call__', return_value=[mock_result]):
            detections = service.detect_fruits(test_image)

            assert len(detections) == 1
            assert detections[0]['class_name'] in service.labels.values()
            assert detections[0]['confidence'] == 0.95
            assert len(detections[0]['bbox']) == 4

    @pytest.mark.asyncio
    async def test_s3_download(self, service):
        """测试 S3 图像下载"""
        # 创建模拟的 S3 响应
        mock_response_body = Mock()

        # 创建测试图像数据
        test_image_pil = Image.new('RGB', (640, 480), color='red')
        img_byte_arr = io.BytesIO()
        test_image_pil.save(img_byte_arr, format='JPEG')
        img_byte_arr.seek(0)

        mock_response_body.read.return_value = img_byte_arr.getvalue()

        mock_response = {
            'Body': mock_response_body
        }

        with patch.object(service.s3_client, 'get_object', return_value=mock_response):
            image = service.download_image_from_s3('test/image.jpg')

            assert image is not None
            assert isinstance(image, np.ndarray)
            assert len(image.shape) == 3
            assert image.shape[2] == 3  # RGB channels

    def test_kafka_message_sending(self, service):
        """测试 Kafka 消息发送"""
        mock_future = AsyncMock()
        mock_future.get.return_value = None

        with patch.object(service.kafka_producer, 'send', return_value=mock_future):
            # 这里需要使用 asyncio.run 因为方法是 async
            asyncio.run(service.send_result_to_kafka(
                'test-upload-123',
                [{'class_name': '苹果', 'confidence': 0.9}],
                'completed'
            ))

            # 验证 Kafka 发送被调用
            service.kafka_producer.send.assert_called_once()

class TestImageProcessingUtils:
    """图像处理工具测试"""

    def test_color_uniformity_calculation(self):
        """测试颜色均匀性计算"""
        from utils.postprocess import calculate_color_uniformity

        # 创建均匀颜色图像
        uniform_image = np.full((100, 100, 3), [120, 150, 80], dtype=np.uint8)
        hsv_uniform = cv2.cvtColor(uniform_image, cv2.COLOR_RGB2HSV)
        uniformity_high = calculate_color_uniformity(hsv_uniform)

        # 创建不均匀颜色图像
        nonuniform_image = np.random.randint(0, 255, (100, 100, 3), dtype=np.uint8)
        hsv_nonuniform = cv2.cvtColor(nonuniform_image, cv2.COLOR_RGB2HSV)
        uniformity_low = calculate_color_uniformity(hsv_nonuniform)

        # 均匀图像应该有更高的均匀性分数
        assert uniformity_high > uniformity_low
        assert 0 <= uniformity_high <= 1
        assert 0 <= uniformity_low <= 1

    def test_surface_defect_analysis(self):
        """测试表面缺陷分析"""
        from utils.postprocess import analyze_surface_defects

        # 创建无缺陷图像 (均匀颜色)
        clean_image = np.full((200, 200, 3), [150, 100, 80], dtype=np.uint8)
        clean_defects = analyze_surface_defects(clean_image)

        # 创建有缺陷图像 (添加线条和斑点)
        defective_image = clean_image.copy()
        cv2.line(defective_image, (50, 50), (150, 50), (0, 0, 0), 3)  # 划痕
        cv2.circle(defective_image, (100, 150), 20, (50, 50, 50), -1)  # 斑点

        defective_defects = analyze_surface_defects(defective_image)

        # 有缺陷图像应该检测到更多缺陷
        assert len(defective_defects) > len(clean_defects)

    def test_size_score_calculation(self):
        """测试尺寸评分计算"""
        from utils.postprocess import calculate_size_score

        # 测试不同尺寸的评分
        small_score = calculate_size_score((50, 50))
        optimal_score = calculate_size_score((100, 100))
        large_score = calculate_size_score((300, 300))

        assert 0 <= small_score <= 1
        assert 0 <= optimal_score <= 1
        assert 0 <= large_score <= 1

        # 最优尺寸应该有最高评分
        assert optimal_score >= small_score
        assert optimal_score >= large_score

class TestModelPerformance:
    """模型性能测试"""

    @pytest.mark.performance
    def test_inference_speed(self, service, test_image):
        """测试推理速度"""
        import time

        # 预热
        for _ in range(5):
            service.detect_fruits(test_image)

        # 测试推理时间
        start_time = time.time()
        num_inferences = 10

        for _ in range(num_inferences):
            service.detect_fruits(test_image)

        end_time = time.time()
        avg_time = (end_time - start_time) / num_inferences

        # 推理时间应该小于 1 秒
        assert avg_time < 1.0

    @pytest.mark.performance
    def test_batch_processing(self, service):
        """测试批处理性能"""
        batch_size = 4
        images = [
            np.random.randint(0, 255, (480, 640, 3), dtype=np.uint8)
            for _ in range(batch_size)
        ]

        # 批处理应该比单个处理更快
        import time

        # 单个处理时间
        start_time = time.time()
        for img in images:
            service.detect_fruits(img)
        single_time = time.time() - start_time

        # 批处理时间 (这里简化，实际需要修改 detect_fruits 支持批处理)
        start_time = time.time()
        for img in images:  # 实际应该是 service.detect_fruits_batch(images)
            service.detect_fruits(img)
        batch_time = time.time() - start_time

        # 记录性能指标
        print(f"Single processing time: {single_time:.3f}s")
        print(f"Batch processing time: {batch_time:.3f}s")

@pytest.mark.integration
class TestSystemIntegration:
    """系统集成测试"""

    @pytest.fixture
    def temp_model_file(self):
        """创建临时模型文件"""
        with tempfile.NamedTemporaryFile(suffix='.pt', delete=False) as f:
            # 创建虚拟模型文件
            f.write(b'fake model data')
            return f.name

    @pytest.mark.slow
    def test_full_pipeline(self, temp_model_file):
        """测试完整的检测流程"""
        # 这个测试需要更多的设置，包括真实的模型文件
        pass

if __name__ == "__main__":
    # 运行测试
    pytest.main([
        __file__,
        "-v",
        "--tb=short",
        "-m", "not slow"  # 排除慢测试
    ])