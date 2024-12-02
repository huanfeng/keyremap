# KeyRemap 键盘映射工具

[English](README_en.md) | [中文](README.md)

KeyRemap 是一个使用 Rust 编写的轻量级键盘映射工具，可以自定义键盘按键映射，提高工作效率。

## 主要特性

- 支持单键映射（如将 Pause 键映射为 Insert 键）
- 支持组合键映射
- 支持鼠标按键映射
- 使用 TOML 配置文件，易于理解和修改
- 低资源占用

## 系统要求

- Windows 操作系统

## 安装方法

1. 从 Release 页面下载最新版本的可执行文件
2. 将可执行文件放置在任意目录下
3. 创建配置文件 `keyremap.toml`

## 使用说明

### 基本用法

1. 先参考 [配置文件示例](#配置文件示例) 创建配置文件，保存在程序所在目录的`keyremap.toml`
2. 如果需要查看具体的按键值，可以运行 `keyremap --listen` 来查看
3. 在命令行下运行 `keyremap` 以测试效果，正常后使用 `keyremap --daemon` 后台运行
4. 开机启动可以使用以下几种方案：

   a. 使用开机启动目录（推荐）
      - 当前用户启动目录：
        ```
        %APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup
        # 或
        C:\Users\用户名\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup
        ```
      - 所有用户启动目录：
        ```
        %ALLUSERSPROFILE%\Microsoft\Windows\Start Menu\Programs\Startup
        # 或
        C:\ProgramData\Microsoft\Windows\Start Menu\Programs\Startup
        ```
      - 在启动目录中创建快捷方式：
      1. 右键 `keyremap.exe` -> 发送到 -> 桌面快捷方式
      2. 右键新创建的快捷方式 -> 属性
      3. 在"目标"后面添加参数：`--daemon`
      4. 将快捷方式移动到上述启动目录之一

   b. 使用注册表
      ```
      # 打开运行对话框（Win + R），输入 regedit，打开注册表编辑器
      # 定位到：HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run
      # 新建字符串值，名称自定义（如 KeyRemap），数据为程序完整路径（如 D:\Tools\keyremap.exe --daemon）
      ```

### 命令行参数

```bash
USAGE:
    keyremap [OPTIONS]

OPTIONS:
    -c, --config <PATH>     指定配置文件路径，默认为程序所在目录的 keyremap.toml
    -v, --verbose           设置日志级别，可重复使用增加详细程度，如 -v 或 -vv
        --logfile           将日志写入文件（keyremap.log）
    -l, --listen            监听模式，显示所有按键事件
        --dump              显示当前配置文件内容
    -d, --daemon            后台模式（Windows），隐藏控制台窗口
    -h, --help              显示帮助信息
    -V, --version           显示版本信息

示例：
# 使用指定的配置文件启动
keyremap -c D:\my_config.toml

# 以监听模式启动，查看按键事件
keyremap --listen

# 以调试模式启动，查看详细日志
keyremap -v

# 以后台模式启动
keyremap --daemon
```

### 配置文件示例

```toml
version = "1.0"
name = "配置1"
comment = "自定义按键映射配置"

[[key_mappings]]
name = "Pause转Insert"
comment = "将Pause键映射为Insert键"
from.key = "Pause"
to.key = "Insert"

[[key_mappings]]
name = "鼠标侧键转Ctrl+W"
from.button.Unknown = 2
to.combination = [
    { key = "ControlLeft" },
    { key = "KeyW" }
]
```

### 配置文件说明

- `version`: 配置文件版本
- `name`: 配置方案名称
- `comment`: 配置说明（可选）
- `key_mappings`: 按键映射列表
  - `name`: 映射名称
  - `enable`: 是否启用（可选，默认为true）
  - `comment`: 映射说明（可选）
  - `from`: 源按键
      - 为按键时，格式为 `key = "键值"`
      - 为鼠标按键时，格式为 `button = "button值"`
  - `to`: 目标按键或组合键
      - 为按键时，格式为 `key = "键值"`
      - 为组合键时，格式为 `combination = [ { key = "键值" }, ... ]`
  - 注意: 如果按键或Button名称不是标准的，会使用 Unknown 数字替代，格式会从 `key = "键值"` 变为 `key.Unknown = 数字`, button 也是同理, 具体以 --listen 输入的值为准

## 开发相关

### 构建项目

```bash
cargo build --release
```

### 依赖项

- rdev: 键盘鼠标事件处理
- serde: 序列化支持
- toml: 配置文件解析
- log: 日志支持
- env_logger: 日志环境配置
- clap: 命令行参数解析
- windows-sys: Windows API 支持

## 许可证

本项目采用 MIT 许可证，详见 [LICENSE](LICENSE) 文件。

## 贡献

欢迎提交 Issue 和 Pull Request！

## 常见问题

1. Q: 程序无法启动？  
   A: 请确保以管理员权限运行程序。

2. Q: 配置文件修改后不生效？  
   A: 请重启程序进行加载。

3. Q: 如何临时禁用某个映射？  
   A: 在对应的映射配置中添加 `enable = false`。
