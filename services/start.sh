#!/bin/bash

# ttyd-rust service startup script

# 默认配置
PORT=7681
INTERFACE="0.0.0.0"
WORKING_DIR="/tmp"
COMMAND="bash"

# 显示帮助信息
show_help() {
    echo "Usage: $0 [OPTIONS]"
    echo "Start ttyd-rust service"
    echo ""
    echo "Options:"
    echo "  -p, --port PORT        Port to listen (default: 7681)"
    echo "  -i, --interface IP     Network interface to bind (default: 0.0.0.0)"
    echo "  -w, --workdir DIR      Working directory (default: /tmp)"
    echo "  -c, --command CMD      Command to execute (default: bash)"
    echo "  -h, --help             Show this help message"
    echo ""
    echo "Example:"
    echo "  $0 -p 8080 -c htop"
    echo "  $0 --port 3000 --command \"python3 -i\""
}

# 解析命令行参数
while [[ $# -gt 0 ]]; do
    case $1 in
        -p|--port)
            PORT="$2"
            shift 2
            ;;
        -i|--interface)
            INTERFACE="$2"
            shift 2
            ;;
        -w|--workdir)
            WORKING_DIR="$2"
            shift 2
            ;;
        -c|--command)
            COMMAND="$2"
            shift 2
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# 检查可执行文件是否存在
if [ ! -f "./ttyd-rust" ]; then
    echo "Error: ttyd-rust executable not found!"
    exit 1
fi

# 设置可执行权限
chmod +x ./ttyd-rust

# 启动服务
echo "Starting ttyd-rust service..."
echo "Port: $PORT"
echo "Interface: $INTERFACE"
echo "Working directory: $WORKING_DIR"
echo "Command: $COMMAND"
echo "Access URL: http://$INTERFACE:$PORT"
echo ""

# 执行程序
./ttyd-rust -p "$PORT" -i "$INTERFACE" -w "$WORKING_DIR" $COMMAND