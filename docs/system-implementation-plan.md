# TerminUS 系统实现与组件规划 (System Implementation & Components Plan)

本规划详细描述了 TerminUS 微内核及其核心系统组件的具体架构、技术选型及分阶段开发计划。

## 1. 微内核核心架构 (Microkernel Core)

TerminUS 内核采用“极简内核”设计，其核心逻辑（Wasm 模块）仅负责最基础的资源仲裁。

### 1.1 核心组件
- **Loader (加载器)**: 解析 `.tep` 包，利用 `wasmtime` 实例化子进程，并根据 `manifest.json` 分配线性内存上限。
- **Capability Manager (能力管理器)**: 维护一个对象表（Object Table），记录每个进程拥有的句柄（如文件句柄、GPU 缓冲区 ID、IPC 端口）。
- **Interrupt/Event Dispatcher (事件分发器)**: 接收宿主机的原始终端事件（鼠标、键盘），并将其路由到当前活跃（Active）的进程或系统服务。

## 2. 系统服务组件 (System Services)

为了保持内核精简，大部分 OS 功能作为独立的“服务进程”运行。

### 2.1 VFS Server (虚拟文件系统服务)
- **技术方案**: 
    - 负责将类 Unix 路径（如 `/home/user/config`）映射到 EVD 的逻辑扇区。
    - 维护全局文件锁和挂载表。
    - **同步机制**: 与宿主机同步时，处理 `disk.evd` 的实时加解密流。

### 2.2 TUI Compositor (TUI 合成器)
- **技术方案**: 
    - 借鉴图形界面的窗口管理器（Window Manager）。
    - 维护一个全局 TUI 渲染缓冲区（字符 + 样式矩阵）。
    - 每个应用拥有自己的“离屏缓冲区”，Compositor 负责将其合成（Alpha Blending 或 Z-order 覆盖）到主终端输出。
    - **交互逻辑**: 拦截鼠标点击坐标，判断点击命中了哪个应用的“窗口”范围，并发送事件。

### 2.3 IPC Server (进程间通信服务)
- **技术方案**: 
    - 提供命名管道 (Named Pipes) 和 共享内存 (Shared Memory) 接口。
    - 支持异步消息队列，用于应用请求系统服务（如 `Requesting GPU Resource`）。

## 3. 应用程序打包规范 (TEP Bundle Spec)

`.tep` 文件夹包含：
1. `bin/main.wasm`: 核心逻辑。
2. `meta/manifest.json`: 定义权限（如 `access: ["/home", "gpu"]`）。
3. `ui/layout.tui`: (可选) 声明式界面描述文件。

## 4. 分步骤开发计划 (Development Phases)

### 阶段 1: 内核启动与首个 TEP 加载 (Kernel Boot & First TEP)
- [ ] 在 Guest 空间实现基础的 `tus_spawn` 系统调用。
- [ ] 实现内核从内存缓冲区（Mock）加载并启动一个简单的 `.wasm` 应用。
- [ ] 验证父子 Wasm 实例间的内存隔离。

### 阶段 2: 显卡驱动与物理窗口对接 (Graphics Driver & Windowing)
- [ ] 在内核中实现 GPU 资源管理器，支持按应用隔离 Buffer ID。
- [ ] 封装 `tus_gpu_open_window` 系统调用，触发宿主机弹出 `winit` 原生窗口。
- [ ] 验证：应用能在保持终端正常交互的同时，在桌面上额外弹出一个实时渲染的 3D 或 2D 窗口。

### 阶段 3: 虚拟文件系统 (VFS) 与 EVD 对接
- [ ] 实现 VFS Server，支持基本的 `ls`, `cd`, `read`, `write` 内部调用。
- [ ] 将 VFS 读写逻辑通过 ABI 对接到宿主机的 `disk.evd` 加密磁盘。
- [ ] 验证：在重启虚拟机后，Guest 内部创建的文件依然存在且被加密。

### 3 阶段: 多窗口 TUI 合成器 (TUI Compositor)
- [ ] 开发 Compositor 服务，支持在单个终端内“分屏”显示两个不同的应用输出。
- [ ] 实现鼠标坐标到应用窗口的映射逻辑。
- [ ] 验证：鼠标拖拽面板边缘可动态调整应用显示区域。

### 阶段 4: TEP 打包与分发工具链
- [ ] 编写一个简单的宿主机工具 `tep-pack`，将 Wasm、JSON 和资源打包为文件夹。
- [ ] 实现内核的“动态安装”功能，从 VHD 镜像中扫描并注册 `.tep`。

### 阶段 5: 系统级 Shell 与 用户空间
- [ ] 开发 TerminUS 默认 Shell，支持类 Unix 命令。
- [ ] 实现基于 Ed25519 的用户登录与磁盘解锁流程。
