当前果蔬分拣系统的前端采用 Vue 3 + TypeScript + Vite，实现了上传流程与结果查询两大页面。目录结构如下：

- `src/`
  - `api/`
    - `upload.ts`：封装上传接口调用，处理 `POST /api/v1/uploads`。
    - `result.ts`：封装结果、统计相关的 API 调用。
  - `components/`
    - `UploadPanel.vue`：图像上传组件，负责文件选择、上传反馈与最近记录列表。
  - `views/`
    - `HomeView.vue`：展示项目简介与主要能力。
    - `ResultsView.vue`：显示历史检测记录、统计看板与详情抽屉。
  - `App.vue`：顶层布局与导航。
  - `main.ts`：应用入口，注册路由与全局样式。
  - `router/index.ts`：定义首页、上传页、结果页三条路由。
  - `style/`：Tailwind 与全局样式文件。

其他文件：
- `vite.config.ts`：Vite 配置。
- `tsconfig*.json`：TypeScript 编译配置。
- `Dockerfile`：前端打包与部署镜像。

核心交互流程：
1. 用户在“图像上传”页选择图片，调用 `uploadApi.uploadFile` 完成上传并在 UI 中展示最近结果。
2. “检测结果”页通过 `resultApi` 加载统计指标与历史记录，支持状态筛选、分页和详情查看。
3. 所有接口地址由环境变量 `VITE_API_BASE_URL` 决定，默认指向后端 `http://localhost:8080/api/v1`。
