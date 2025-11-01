# ttyd-rust Service

这是一个独立的终端共享服务，基于Rust开发，包含所有必要的依赖文件。

## 目录结构

- `ttyd-rust`: 主程序文件（4.4MB，包含所有静态资源）
- `start.sh`: 启动脚本
- `source/`: 源代码目录
  - `src/`: Rust源代码
  - `Cargo.toml`: 项目依赖配置
  - `Cargo.lock`: 依赖版本锁定
  - `README.md`: 项目说明文档

## 文件说明

- `ttyd-rust`: 主程序文件（4.4MB，包含所有静态资源）
- `start.sh`: 启动脚本

## 快速开始

### Linux/macOS

```bash
# 添加执行权限
chmod +x ttyd-rust start.sh

# 使用默认配置启动（端口7681）
./start.sh

# 或直接运行程序
./ttyd-rust
```

### Windows

```cmd
# 直接运行程序
ttyd-rust.exe
```

## 使用参数

```bash
# 指定端口
./start.sh -p 8080

# 指定绑定地址
./start.sh -i 127.0.0.1

# 指定工作目录
./start.sh -w /home/user

# 运行特定命令
./start.sh -c htop
./start.sh -c "python3 -i"
./start.sh -c vim

# 组合使用
./start.sh -p 3000 -c htop
```

## 访问终端

启动后，在浏览器中访问以下地址：

```
http://localhost:7681
```

如果指定了不同的端口或IP，请相应调整URL。

## 编译源代码

如果需要从源代码编译：

```bash
cd source
cargo build --release
```

编译后的可执行文件位于 `source/target/release/ttyd-rust`

## 特性

- 高性能异步架构（基于Tokio）
- 完整的WebSocket支持
- xterm.js终端渲染（包含WebGL加速）
- 跨平台支持（Linux/macOS/Windows）
- 单文件分发（无需额外依赖）
- 支持所有TUI程序（htop、vim、tmux等）

## 系统要求

- Linux/macOS/Windows 10+
- Rust 1.70+ (如需编译源代码)
- 无需安装其他依赖