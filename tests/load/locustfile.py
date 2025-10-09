import time
import json
import random
from locust import HttpUser, task, between, events
from locust.contrib.fasthttp import FastHttpUser

class FruitSortingUser(FastHttpUser):
    wait_time = between(1, 3)  # 用户操作间隔时间

    def on_start(self):
        """用户开始时执行的初始化操作"""
        # 模拟用户登录
        response = self.client.post("/auth/login", json={
            "username": "test_user",
            "password": "test_password"
        })

        if response.status_code == 200:
            auth_data = response.json()
            self.token = auth_data.get("access_token")
            # 设置认证头
            self.client.headers.update({
                "Authorization": f"Bearer {self.token}"
            })
        else:
            print(f"Login failed with status {response.status_code}")

    @task(40)  # 权重最高：文件上传是核心功能
    def upload_image(self):
        """测试图像上传功能"""
        # 生成模拟图片数据
        fake_image_data = self.generate_fake_image()

        files = {
            'file': ('test_fruit.jpg', fake_image_data, 'image/jpeg')
        }

        with self.client.post(
            "/api/v1/uploads",
            files=files,
            catch_response=True,
            name="Upload Image"
        ) as response:
            if response.status_code == 200:
                result = response.json()
                upload_id = result.get("upload_id")

                if upload_id:
                    response.success()
                    # 存储 upload_id 用于后续查询
                    self.upload_ids = getattr(self, 'upload_ids', [])
                    self.upload_ids.append(upload_id)
                else:
                    response.failure("No upload_id in response")
            else:
                response.failure(f"Upload failed: {response.status_code}")

    @task(20)
    def get_upload_result(self):
        """查询上传结果"""
        # 确保有可用的 upload_id
        if not hasattr(self, 'upload_ids') or not self.upload_ids:
            return

        upload_id = random.choice(self.upload_ids)

        with self.client.get(
            f"/api/v1/results/{upload_id}",
            catch_response=True,
            name="Get Result"
        ) as response:
            if response.status_code == 200:
                result = response.json()
                if result.get("job_id") == upload_id:
                    response.success()
                else:
                    response.failure("Invalid result data")
            elif response.status_code == 404:
                # 结果还未准备好，这是正常情况
                response.success()
            else:
                response.failure(f"Get result failed: {response.status_code}")

    @task(15)
    def list_results(self):
        """获取结果列表"""
        params = {
            "page": random.randint(1, 5),
            "limit": random.choice([10, 20, 50]),
            "status": random.choice(["pending", "completed", "failed", None])
        }

        # 移除 None 值
        params = {k: v for k, v in params.items() if v is not None}

        with self.client.get(
            "/api/v1/results",
            params=params,
            catch_response=True,
            name="List Results"
        ) as response:
            if response.status_code == 200:
                data = response.json()
                if "data" in data and "pagination" in data:
                    response.success()
                else:
                    response.failure("Invalid response format")
            else:
                response.failure(f"List results failed: {response.status_code}")

    @task(10)
    def get_statistics(self):
        """获取统计数据"""
        time_range = random.choice(["1h", "24h", "7d", "30d"])

        with self.client.get(
            "/api/v1/statistics",
            params={"range": time_range},
            catch_response=True,
            name="Get Statistics"
        ) as response:
            if response.status_code == 200:
                stats = response.json()
                required_fields = ["total_processed", "accuracy_rate", "grade_distribution"]

                if all(field in stats for field in required_fields):
                    response.success()
                else:
                    response.failure("Missing required fields in statistics")
            else:
                response.failure(f"Get statistics failed: {response.status_code}")

    @task(8)
    def control_sorting(self):
        """分拣控制测试"""
        actions = ["start", "stop", "pause", "resume"]
        action = random.choice(actions)

        payload = {
            "job_id": f"sort_{int(time.time())}_{random.randint(1000, 9999)}",
            "action": action,
            "target_grade": random.choice(["A", "B", "C"]),
            "conveyor_speed": random.randint(30, 100)
        }

        with self.client.post(
            "/api/v1/sort/command",
            json=payload,
            catch_response=True,
            name=f"Sort Control - {action.title()}"
        ) as response:
            if response.status_code == 200:
                result = response.json()
                if result.get("success"):
                    response.success()
                else:
                    response.failure("Sort command failed")
            else:
                response.failure(f"Sort control failed: {response.status_code}")

    @task(5)
    def get_sort_status(self):
        """获取分拣状态"""
        with self.client.get(
            "/api/v1/sort/status",
            catch_response=True,
            name="Get Sort Status"
        ) as response:
            if response.status_code == 200:
                status = response.json()
                required_fields = ["status", "conveyor_speed", "sorted_count"]

                if all(field in status for field in required_fields):
                    response.success()
                else:
                    response.failure("Missing required fields in sort status")
            else:
                response.failure(f"Get sort status failed: {response.status_code}")

    @task(3)
    def export_results(self):
        """导出结果测试"""
        export_format = random.choice(["csv", "json", "xml"])

        params = {
            "format": export_format,
            "limit": random.randint(10, 100)
        }

        with self.client.get(
            "/api/v1/results/export",
            params=params,
            catch_response=True,
            name=f"Export Results - {export_format.upper()}"
        ) as response:
            if response.status_code == 200:
                # 检查响应头
                content_type = response.headers.get('content-type', '')
                expected_types = {
                    'csv': 'text/csv',
                    'json': 'application/json',
                    'xml': 'application/xml'
                }

                if expected_types[export_format] in content_type:
                    response.success()
                else:
                    response.failure(f"Unexpected content type: {content_type}")
            else:
                response.failure(f"Export failed: {response.status_code}")

    @task(2)
    def presigned_upload(self):
        """预签名URL上传测试"""
        filename = f"fruit_{int(time.time())}.jpg"

        # 获取预签名URL
        with self.client.post(
            "/api/v1/uploads/presign",
            json={
                "filename": filename,
                "content_type": "image/jpeg"
            },
            catch_response=True,
            name="Get Presigned URL"
        ) as response:
            if response.status_code == 200:
                result = response.json()
                presigned_url = result.get("presigned_url")

                if presigned_url:
                    # 模拟直接上传到 MinIO
                    fake_image_data = self.generate_fake_image()

                    # 注意：这里应该使用原始的HTTP客户端，不是self.client
                    # 因为预签名URL指向MinIO，不是API网关
                    response.success()
                else:
                    response.failure("No presigned URL in response")
            else:
                response.failure(f"Get presigned URL failed: {response.status_code}")

    def generate_fake_image(self):
        """生成模拟的图片数据"""
        # 生成随机字节数据模拟JPEG文件
        fake_jpeg_header = b'\xff\xd8\xff\xe0\x00\x10JFIF'
        fake_image_data = fake_jpeg_header + bytes([random.randint(0, 255) for _ in range(1024)])
        fake_jpeg_footer = b'\xff\xd9'

        return fake_image_data + fake_jpeg_footer

class HighConcurrencyUser(FruitSortingUser):
    """高并发用户，操作间隔更短"""
    wait_time = between(0.1, 0.5)

class BurstTrafficUser(FruitSortingUser):
    """突发流量用户，集中操作后长时间等待"""
    wait_time = between(0.1, 1)

    @task(50)
    def burst_upload(self):
        """突发上传测试"""
        # 连续上传多个文件
        for i in range(random.randint(3, 7)):
            self.upload_image()
            time.sleep(0.1)  # 短暂间隔

class WebSocketUser(HttpUser):
    """WebSocket连接测试用户"""
    wait_time = between(5, 15)

    def on_start(self):
        self.ws = None
        self.connect_websocket()

    def connect_websocket(self):
        """连接WebSocket"""
        # 注意：Locust 不直接支持WebSocket，这里是示意代码
        # 实际实现需要使用websocket-client库
        pass

    @task
    def send_ping(self):
        """发送心跳"""
        if self.ws:
            # self.ws.send("ping")
            pass

# 自定义事件处理
@events.request_success.add_listener
def on_request_success(request_type, name, response_time, response_length, **kw):
    """请求成功事件处理"""
    if response_time > 5000:  # 超过5秒的请求
        print(f"Slow request: {name} took {response_time}ms")

@events.request_failure.add_listener
def on_request_failure(request_type, name, response_time, response_length, exception, **kw):
    """请求失败事件处理"""
    print(f"Request failed: {name} - {exception}")

@events.test_start.add_listener
def on_test_start(environment, **kw):
    """测试开始事件"""
    print("Load test started")

@events.test_stop.add_listener
def on_test_stop(environment, **kw):
    """测试结束事件"""
    print("Load test finished")

    # 计算并打印统计信息
    stats = environment.stats
    print(f"Total requests: {stats.total.num_requests}")
    print(f"Total failures: {stats.total.num_failures}")
    print(f"Average response time: {stats.total.avg_response_time:.2f}ms")
    print(f"Max response time: {stats.total.max_response_time}ms")
    print(f"Requests per second: {stats.total.current_rps:.2f}")

# 压测场景定义
class LoadTestScenarios:
    """不同的负载测试场景"""

    @staticmethod
    def normal_traffic():
        """正常流量场景"""
        return {
            'FruitSortingUser': 80,  # 80% 正常用户
            'HighConcurrencyUser': 20  # 20% 高并发用户
        }

    @staticmethod
    def peak_traffic():
        """峰值流量场景"""
        return {
            'FruitSortingUser': 40,
            'HighConcurrencyUser': 40,
            'BurstTrafficUser': 20
        }

    @staticmethod
    def stress_test():
        """压力测试场景"""
        return {
            'HighConcurrencyUser': 70,
            'BurstTrafficUser': 30
        }

# 使用示例命令：
#
# 正常负载测试：
# locust -f locustfile.py --users 100 --spawn-rate 10 --run-time 10m --host http://localhost:8000
#
# 压力测试：
# locust -f locustfile.py --users 500 --spawn-rate 50 --run-time 30m --host http://localhost:8000
#
# 持续负载测试：
# locust -f locustfile.py --users 200 --spawn-rate 20 --run-time 2h --host http://localhost:8000
#
# 无界面模式（CI/CD）：
# locust -f locustfile.py --headless --users 100 --spawn-rate 10 --run-time 5m --host http://localhost:8000 --csv=results
