use axum::{
    body::Body,
    http::{header, HeaderValue, Response, StatusCode},
    response::IntoResponse,
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "static/"]
struct StaticFiles;

pub async fn static_handler(uri: &str) -> impl IntoResponse {
    // 处理路径
    let path = match uri {
        "/xterm.min.css" => "js/xterm.min.css",
        _ if uri.starts_with("/js/") => &uri[1..],  // "/js/file.js" -> "js/file.js"
        _ if uri.starts_with("/css/") => &uri[1..], // "/css/file.css" -> "css/file.css"
        _ => {
            // 对于其他路径，尝试添加js前缀
            if uri.ends_with(".js") || uri.ends_with(".css") {
                if !uri.starts_with('/') {
                    &format!("js/{}", uri)[..]
                } else {
                    &format!("js{}", uri)[..]
                }
            } else {
                uri
            }
        }
    };
    
    match StaticFiles::get(path) {
        Some(content) => {
            let mime_type = mime_guess::from_path(path).first_or_octet_stream();
            Response::builder()
                .status(StatusCode::OK)
                .header(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str(mime_type.as_ref()).unwrap(),
                )
                .body(Body::from(content.data))
                .unwrap()
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from(format!("File not found: {}", path)))
            .unwrap(),
    }
}

pub const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>ttyd-rust - Terminal</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        body, html {
            height: 100%;
            overflow: hidden;
            background: #000;
            font-family: 'Courier New', monospace;
        }
        #terminal {
            width: 100%;
            height: 100%;
            padding: 10px;
        }
        #error-message {
            color: red;
            padding: 20px;
            font-family: 'Courier New', monospace;
        }
    </style>
    <link rel="stylesheet" href="/xterm.min.css" />
</head>
<body>
    <div id="terminal"></div>
    <div id="error-message" style="display: none;"></div>
    <script src="/js/xterm.min.js"></script>
    <script src="/js/addon-fit.min.js"></script>
    <script src="/js/addon-webgl.min.js"></script>
    <script>
        // 兼容性检查函数
        function checkBrowserCompatibility() {
            var isCompatible = true;
            var errors = [];
            
            // 检查基本API支持
            if (!window.WebSocket) {
                errors.push("WebSocket API not supported");
                isCompatible = false;
            }
            
            if (!window.TextEncoder || !window.TextDecoder) {
                errors.push("TextEncoder/TextDecoder not supported");
                isCompatible = false;
            }
            
            if (!window.JSON) {
                errors.push("JSON not supported");
                isCompatible = false;
            }
            
            if (!Array.prototype.forEach) {
                errors.push("Array.forEach not supported");
                isCompatible = false;
            }
            
            return {
                isCompatible: isCompatible,
                errors: errors
            };
        }
        
        // 显示错误信息
        function showError(message) {
            var errorDiv = document.getElementById('error-message');
            var terminalDiv = document.getElementById('terminal');
            
            if (errorDiv && terminalDiv) {
                errorDiv.style.display = 'block';
                errorDiv.innerHTML = 'Error: ' + message;
                terminalDiv.style.display = 'none';
            } else {
                document.body.innerHTML = '<div id="error-message">Error: ' + message + '</div>';
            }
            
            console.error(message);
        }
        
        // 确保DOM加载完成后再执行
        function initTerminal() {
            // 兼容性检查
            var compatibility = checkBrowserCompatibility();
            if (!compatibility.isCompatible) {
                showError('Browser compatibility issues detected: ' + compatibility.errors.join(', '));
                return;
            }
            
            // 检查xterm.js库是否已加载
            if (typeof Terminal === 'undefined') {
                showError('Terminal library not loaded. Please check your network connection and refresh the page.');
                return;
            }
            
            // 兼容性更好的变量声明
            var term;
            var fitAddon;
            var ws;
            var inputBuffer = [];
            var sendTimer = null;
            
            try {
                // 检查terminal元素是否存在
                var terminalElement = document.getElementById('terminal');
                if (!terminalElement) {
                    showError('Terminal element not found');
                    return;
                }
                
                // 初始化终端
                term = new Terminal({
                    cursorBlink: true,
                    fontFamily: 'Courier New, monospace',
                    fontSize: 14,
                    theme: {
                        background: '#000000',
                        foreground: '#ffffff'
                    }
                });
                
                // 加载Fit插件
                try {
                    if (typeof FitAddon !== 'undefined' && FitAddon.FitAddon) {
                        fitAddon = new FitAddon.FitAddon();
                        term.loadAddon(fitAddon);
                    } else {
                        console.warn('Fit addon not available');
                    }
                } catch (e) {
                    console.warn('Fit addon failed to load:', e);
                }
                
                // 打开终端
                term.open(terminalElement);
                
                // 调整终端大小
                if (fitAddon) {
                    try {
                        fitAddon.fit();
                    } catch (e) {
                        console.warn('Fit failed:', e);
                    }
                }
                
                // 尝试加载WebGL插件（可选）
                try {
                    if (typeof WebglAddon !== 'undefined' && WebglAddon.WebglAddon) {
                        var webglAddon = new WebglAddon.WebglAddon();
                        term.loadAddon(webglAddon);
                    } else {
                        console.warn('WebGL addon not available');
                    }
                } catch (e) {
                    console.warn('WebGL addon failed to load (optional feature):', e);
                }
                
                // 建立WebSocket连接
                var protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
                ws = new WebSocket(protocol + '//' + window.location.host + '/ws');
                ws.binaryType = 'arraybuffer';
                
                // 发送输入数据的函数
                var flushInput = function() {
                    if (!ws || ws.readyState !== WebSocket.OPEN || inputBuffer.length === 0) {
                        return;
                    }
                    
                    var totalLen = 0;
                    for (var i = 0; i < inputBuffer.length; i++) {
                        totalLen += inputBuffer[i].length;
                    }
                    
                    var msg = new Uint8Array(totalLen + 1);
                    msg[0] = '0'.charCodeAt(0);
                    
                    var offset = 1;
                    for (var j = 0; j < inputBuffer.length; j++) {
                        var data = inputBuffer[j];
                        for (var k = 0; k < data.length; k++) {
                            msg[offset++] = data.charCodeAt(k);
                        }
                    }
                    
                    try {
                        ws.send(msg);
                        inputBuffer = [];
                    } catch (e) {
                        console.error('Failed to send data:', e);
                    }
                };
                
                // 处理终端输入
                term.onData(function(data) {
                    if (ws && ws.readyState === WebSocket.OPEN) {
                        inputBuffer.push(data);
                        
                        if (sendTimer) {
                            clearTimeout(sendTimer);
                        }
                        
                        sendTimer = setTimeout(flushInput, 10);
                    }
                });
                
                // WebSocket连接成功
                ws.onopen = function() {
                    console.log('WebSocket connected');
                    term.focus();
                    
                    // 发送初始尺寸信息
                    if (term.cols && term.rows) {
                        var initMsg = JSON.stringify({
                            columns: term.cols,
                            rows: term.rows
                        });
                        
                        try {
                            var encoder = new TextEncoder();
                            var initBytes = encoder.encode(initMsg);
                            ws.send(initBytes.buffer);
                            console.log('Init message sent as binary');
                        } catch (e) {
                            console.error('Failed to send init message:', e);
                        }
                    }
                };
                
                // 处理WebSocket消息
                ws.onmessage = function(event) {
                    if (!event.data) return;
                    
                    try {
                        var data = new Uint8Array(event.data);
                        if (data.length === 0) return;
                        
                        var cmd = String.fromCharCode(data[0]);
                        var payload = data.slice(1);
                        
                        switch (cmd) {
                            case '0':
                                // 终端输出
                                try {
                                    term.write(payload);
                                } catch (e) {
                                    console.error('Failed to write to terminal:', e);
                                }
                                break;
                            case '1':
                                // 标题更新
                                try {
                                    var title = new TextDecoder().decode(payload);
                                    document.title = title;
                                } catch (e) {
                                    console.error('Failed to decode title:', e);
                                }
                                break;
                            case '2':
                                // 忽略
                                break;
                            default:
                                console.warn('Unknown command:', cmd);
                        }
                    } catch (e) {
                        console.error('Failed to process message:', e);
                    }
                };
                
                // WebSocket错误处理
                ws.onerror = function(error) {
                    console.error('WebSocket error:', error);
                    try {
                        if (term) {
                            term.write('\r\n\x1b[31mWebSocket connection error\x1b[0m\r\n');
                        }
                    } catch (e) {
                        console.error('Failed to write error to terminal:', e);
                    }
                };
                
                // WebSocket关闭处理
                ws.onclose = function() {
                    console.log('WebSocket closed');
                    try {
                        if (term) {
                            term.write('\r\n\x1b[33mConnection closed\x1b[0m\r\n');
                        }
                    } catch (e) {
                        console.error('Failed to write close message to terminal:', e);
                    }
                };
                
                // 窗口大小调整处理
                window.addEventListener('resize', function() {
                    if (fitAddon) {
                        try {
                            fitAddon.fit();
                        } catch (e) {
                            console.warn('Fit failed on resize:', e);
                        }
                    }
                    
                    if (ws && ws.readyState === WebSocket.OPEN && term.cols && term.rows) {
                        try {
                            var resizeMsg = new TextEncoder().encode(
                                '1' + JSON.stringify({ columns: term.cols, rows: term.rows })
                            );
                            ws.send(resizeMsg);
                        } catch (e) {
                            console.error('Failed to send resize message:', e);
                        }
                    }
                });
                
            } catch (e) {
                console.error('Failed to initialize terminal:', e);
                showError('Failed to initialize terminal (' + e.message + ')');
            }
        }
        
        // DOM加载完成后初始化终端
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', initTerminal);
        } else {
            // DOM已经加载完成
            initTerminal();
        }
    </script>
</body>
</html>
"#;

pub async fn index_handler() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, HeaderValue::from_static("text/html"))
        .body(Body::from(INDEX_HTML))
        .unwrap()
}

pub async fn token_handler() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        )
        .body(Body::from(r#"{"token":""}"#))
        .unwrap()
}