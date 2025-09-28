# Copilot Instructions for guoyan-zhizao Frontend

## 项目架构概览
- 本项目为 Vue 3 + TypeScript 前端，采用 Vite 构建，目录结构清晰分为 `src/components`、`src/views`、`src/api`、`src/router`、`src/stores` 等。
- 主要页面通过 `src/views` 实现，功能性组件在 `src/components`。
- 路由由 `src/router/index.ts` 管理，所有页面均为懒加载。
- 状态管理使用 Pinia（见 `src/stores`）。
- 样式采用 Tailwind CSS（见 `src/style/tailwind.css`）。
- 与后端交互通过 axios 封装在 `src/api`，WebSocket 通信集中在 `src/websocket.ts`。

## 关键开发流程
- 启动开发环境：`pnpm dev` 或 `npm run dev`，默认端口 3000。
- 构建生产包：`pnpm build` 或 `npm run build`，输出至 `dist/`。
- 代码检查：`pnpm lint` 或 `npm run lint`，自动修复格式。
- 类型检查：`pnpm type-check` 或 `npm run type-check`。
- Docker 构建：多阶段，详见 `Dockerfile`，支持环境变量注入和 Nginx 部署。

## 约定与模式
- 所有 API 地址、WebSocket 地址均通过环境变量注入（见 `Dockerfile` 和 Vite 配置），开发环境默认指向 `localhost:8000`。
- WebSocket 统一管理，支持断线重连、心跳、消息分发，详见 `src/websocket.ts`，如需扩展新类型只需新增 hook。
- API 封装采用 RESTful 风格，所有接口均在 `src/api` 下按功能拆分（如 `result.ts`, `upload.ts`），并返回强类型数据。
- 组件通信优先使用 props/emits，复杂状态用 Pinia。
- 路由懒加载，页面按需加载，避免首屏性能损耗。
- 生产环境通过 Nginx 代理 `/api` 和 `/ws`，前后端分离，静态资源缓存 1 年。

## 重要文件/目录
- `src/api/`：所有后端接口封装，按业务拆分。
- `src/websocket.ts`：WebSocket 管理与 hook 封装。
- `src/components/`：功能性组件，复用性强。
- `src/views/`：页面级组件。
- `src/router/index.ts`：路由配置。
- `Dockerfile`：生产部署与环境变量注入。
- `vite.config.ts`：构建与代理配置。

## 典型用例示例
- 新增 API：在 `src/api/` 新建文件，封装 axios 方法并导出。
- 新增 WebSocket 服务：在 `src/websocket.ts` 按模板扩展 hook。
- 新增页面：在 `src/views/` 新建 Vue 文件，并在 `router/index.ts` 注册路由。
- 新增组件：在 `src/components/` 新建 Vue 文件，按需引入。

---
如有特殊约定或未覆盖场景，请补充说明。建议 AI agent 变更前先查阅相关目录和文件，保持风格一致。
