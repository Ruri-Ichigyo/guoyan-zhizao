import io
import random
from locust import task, between
from locust.contrib.fasthttp import FastHttpUser


def make_fake_image() -> bytes:
    # 简单生成固定大小的伪 JPEG 数据
    header = b"\xff\xd8\xff\xe0\x00\x10JFIF"
    body = bytes(random.randint(0, 255) for _ in range(2048))
    footer = b"\xff\xd9"
    return header + body + footer


class FruitSortingUser(FastHttpUser):
    wait_time = between(1, 3)

    def on_start(self):
        self.upload_ids: list[str] = []

    @task(5)
    def upload_image(self):
        files = {
            "file": ("locust.jpg", make_fake_image(), "image/jpeg")
        }

        with self.client.post(
            "/api/v1/uploads",
            files=files,
            name="Upload Image",
            catch_response=True,
        ) as response:
            if response.status_code == 200:
                data = response.json()
                upload_id = data.get("upload_id")
                if upload_id:
                    self.upload_ids.append(upload_id)
                    response.success()
                else:
                    response.failure("missing upload_id")
            else:
                response.failure(f"unexpected status {response.status_code}")

    @task(3)
    def list_results(self):
        params = {
            "page": random.randint(1, 3),
            "limit": random.choice([5, 10, 20]),
            "status": random.choice(["completed", "processing", "failed", None]),
        }
        params = {k: v for k, v in params.items() if v is not None}

        self.client.get(
            "/api/v1/results",
            params=params,
            name="List Results",
        )

    @task(2)
    def fetch_detail(self):
        if not self.upload_ids:
            return

        upload_id = random.choice(self.upload_ids)
        self.client.get(
            f"/api/v1/results/{upload_id}",
            name="Get Result Detail",
        )

    @task(1)
    def fetch_statistics(self):
        self.client.get(
            "/api/v1/statistics",
            name="Get Statistics",
        )


class SmokeUser(FruitSortingUser):
    wait_time = between(3, 5)
