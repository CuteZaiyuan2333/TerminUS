# TerminUS 核心架构方案 (Architecture Design)

## 1. 系统概览 (System Overview)

TerminUS (Terminal Unified System) 是一款基于 Wasm64 虚拟机的微内核 TUI 操作系统。它将物理机资源（GPU、磁盘、终端）抽象为标准的系统调用，并通过高性能的 TUI 引擎为用户提供沉浸式的命令行交互体验。

## 2. 微内核设计 (Microkernel Architecture)

由于运行在 Wasm 虚拟机中，TerminUS 的内核本身是一个 Wasm 模块（Kernel Space），它负责管理其他 Wasm 应用实例（User Space）。

### 2.1 核心职责

- **进程生命周期管理**: 利用 Wasmtime 的 `Linker` 动态创建子进程实例。
- **资源抽象**: 将宿主机的 `vm_storage` 和 `vm_terminal` 封装为更高级的系统接口（Syscalls）。
- **进程间通信 (IPC)**: 通过 Wasm 的 `Memory.shared` 或 `Table` 实现进程间的消息总线。
- **权限管理**: 控制子进程对 VHD 加密磁盘不同目录的访问权限。

## 3. 应用程序格式 (.tep Bundle)

TerminUS 借鉴 macOS 的 .app 设计，所有应用程序以名为 `<AppName>.tep` (Terminus Executable Program) 的文件夹形式存在。

### 3.1 .tep 目录结构

- `main.wasm`: 核心可执行逻辑（符合 TerminUS 应用 ABI）。
- `manifest.json`: 应用元数据（名称、版本、权限需求、图标、入口）。
- `resources/`: 存放该应用所需的静态资源、TUI 布局文件等。
- `data/`: 应用的私有持久化数据存储区。

## 4. 虚拟文件系统 (VFS)

## 4. 虚拟文件系统 (VFS)
TerminUS 对用户展现类 Unix 的文件层级，并底层采用“双盘并行挂载”机制。

### 4.1 层次结构映射
- **系统层 (System EVD - ReadOnly)**:
    - `/`: 根目录引导区。
    - `/bin`: 核心系统指令。
    - `/lib`: 标准库模块。
    - `/etc/defaults`: 系统默认配置。
- **数据层 (User EVD - ReadWrite)**:
    - `/home`: 用户家目录。
    - `/apps`: 用户安装的 `.tep` 应用包。
    - `/etc`: 用户自定义配置（通过 Overlay 覆盖或软链接）。
    - `/var`: 运行时状态与日志。

- `/drives`: 所有安装的、除了系统盘之外的 `.evd` 加密硬盘的展示位置。

## 5. 图形与显示子系统 (Graphics & Display Subsystem)
TerminUS 将 `wgpu` 视为核心硬件，内核作为“驱动层”管理对显卡的访问。

### 5.1 显卡驱动模型
- **资源仲裁**: 内核负责分配显存空间，并确保各进程间的着色器任务互不干扰。
- **窗口管理器**: TEP 应用可申请创建“独立物理窗口”。内核通过系统调用指示宿主机弹出物理窗口，并将该应用渲染管线的输出直接定向到该窗口。
- **计算加速**: 系统服务（如加密存储、TUI 合成器）可优先申请 GPU 计算配额。

## 6. 系统调用设计 (TerminUS ABI)

- `tus_spawn(tep_path: str)`: 启动一个 TEP 应用。
- `tus_ipc_send(pid: u32, msg: ptr)`: 发送 IPC 消息。
- `tus_fs_open/read/write`: 统一的文件 IO 接口。
- `tus_gpu_request`: 申请 GPU 计算切片。

