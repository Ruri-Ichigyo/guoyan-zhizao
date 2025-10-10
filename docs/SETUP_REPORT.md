# 项目搭建核对报告

本报告汇总当前仓库的整理结果，记录核心目录、脚本与文档的位置，便于团队快速理解项目结构并在后续迭代中保持一致性。

## 1. 本次整理概览
- 统一根目录结构：将启动脚本集中到 `scripts/`，将快速入门与调试记录移入 `docs/`，减少根目录散落文件。
- 对照当前架构（前端 + Rust 后端 + FastAPI 推理服务）核对 `docker-compose.yml` 与核心配置，确认依赖关系清晰。
- 更新主文档与脚本提示，确保所有说明示例均使用 `./scripts/start.sh` 与 `./scripts/dev-start.sh`。
- 复核 `docs/PROJECT_GUIDE.md`、`docs/QUICKSTART.md` 等资料之间的链接与内容，保持信息来源一致。
- 对齐后端新增的 JWT 认证与默认管理员逻辑，同步 API 与部署文档，确保首次登录流程清晰。

## 2. 目录速览

```
.
├── ai/                # FastAPI + PyTorch 推理服务
├── backend/           # Rust (Axum) 后端服务
├── docs/              # 项目文档、部署与算法说明
├── frontend/          # Vue 3 前端应用
├── scripts/           # 环境验证与启动脚本
├── tests/             # 单元、集成、负载与 E2E 测试样例
├── docker-compose.yml # 一体化部署入口
└── .env.example       # 环境变量示例
```

> 详细的模块说明参见 `docs/PROJECT_GUIDE.md`。

## 3. 关键文件清单

| 类型 | 路径 | 说明 |
| --- | --- | --- |
| 环境示例 | `.env.example` | 默认数据库、Redis、前端构建参数，可按需覆盖 |
| Docker 栈 | `docker-compose.yml` | 定义 Postgres、Redis、AI、后端、前端服务及健康检查 |
| 启动脚本 | `scripts/start.sh` | 构建并启动整套容器服务，包含健康检查等待逻辑 |
| 开发脚本 | `scripts/dev-start.sh` | 本地开发辅助，检测依赖并可选启动数据库与缓存 |
| 结构校验 | `scripts/verify-setup.sh` | 快速检查核心目录/文件与本地依赖是否齐全 |
| 快速上手 | `docs/QUICKSTART.md` | 两种启动方式与常见问题说明 |
| 架构指南 | `docs/PROJECT_GUIDE.md` | 全局架构、模块职责、配置说明 |
| API 手册 | `docs/API.md` | REST 接口定义与请求/响应示例 |
| 认证逻辑 | `backend/src/auth.rs` | JWT 生成/校验、角色授权及默认管理员初始化 |

## 4. 启动与开发

### Docker 一键启动
```bash
cp .env.example .env
./scripts/start.sh
```
- 服务启动后：前端 `http://localhost:3000`，后端 API `http://localhost:8080/api/v1`，AI 服务 `http://localhost:8002/health`。
- 默认管理员凭证由 `.env` 中的 `ADMIN_USERNAME` / `ADMIN_PASSWORD` 指定，首次登录可直接访问 `/login` 页面。
- 常用运维命令：`docker compose ps`、`docker compose logs -f backend`、`docker compose down`。

### 本地开发模式
```bash
./scripts/dev-start.sh  # 可选：自动启动 postgres/redis
```
随后按需手动启动：
- Rust 后端：在 `backend/` 执行 `cargo run`（需设置 `DATABASE_URL`、`REDIS_URL`、`AI_SERVICE_URL`，可选覆盖 `JWT_SECRET`、`ADMIN_USERNAME`、`ADMIN_PASSWORD`）。
- FastAPI AI 服务：在 `ai/` 中执行 `uvicorn main:app --reload --host 0.0.0.0 --port 8002`。
- Vue 前端：在 `frontend/` 中执行 `npm install && npm run dev`。

## 5. 验证清单
- `scripts/verify-setup.sh`：检查核心目录、关键文件与三大语言工具链是否可用。
- `cargo test` / `pytest` / `vitest`：分别对应后端、AI 服务与前端单元测试（详见 `tests/` 目录）。
- `docker compose ps`：确认容器化部署的健康状态。
- `curl -X POST http://localhost:8080/api/v1/auth/login ...`：验证默认管理员是否可获取 JWT。

## 6. 后续建议
1. 依据 `docs/PROJECT_GUIDE.md` 定期审视文档是否与实现同步，避免信息漂移。
2. 扩充 `scripts/` 中的自动化脚本，例如数据迁移或日志收集，保持命令入口统一。
3. 对 `tests/` 目录下的示例进行裁剪与更新，使其完全契合当前精简架构。
4. 若引入新的独立文档或脚本，优先放置于 `docs/`、`scripts/` 等目录中，以延续结构化约定。
