# KeyRemap Keyboard Mapping Tool

[English](README_en.md) | [中文](README.md)

KeyRemap is a lightweight keyboard mapping tool written in Rust that allows you to customize keyboard key mappings to improve work efficiency.

## Main Features

- Support single key mapping (e.g., mapping Pause key to Insert key)
- Support combination key mapping
- Support mouse button mapping
- Use TOML configuration file, easy to understand and modify
- Low resource usage

## System Requirements

- Windows operating system

## Installation

1. Download the latest executable from the Release page
2. Place the executable in any directory
3. Create a configuration file `keyremap.toml`

## Usage

### Basic Usage

1. First, create a configuration file by referring to the [Configuration Example](#configuration-example), save it as `keyremap.toml` in the program directory
2. If you need to check specific key values, run `keyremap --listen`
3. Run `keyremap` in command line to test, then use `keyremap --daemon` for background running
4. For startup with Windows, you can use the following methods:

   a. Use startup folder (recommended)
      - Current user startup directory:
        ```
        %APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup
        # or
        C:\Users\username\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup
        ```
      - All users startup directory:
        ```
        %ALLUSERSPROFILE%\Microsoft\Windows\Start Menu\Programs\Startup
        # or
        C:\ProgramData\Microsoft\Windows\Start Menu\Programs\Startup
        ```
      - Create shortcut in startup directory:
      1. Right-click `keyremap.exe` -> Send to -> Desktop (create shortcut)
      2. Right-click the new shortcut -> Properties
      3. Add parameter after "Target": `--daemon`
      4. Move the shortcut to one of the startup directories above

   b. Use registry
      ```
      # Open Run dialog (Win + R), type regedit, open Registry Editor
      # Navigate to: HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run
      # Create new string value, name as you like (e.g., KeyRemap), data is full program path (e.g., D:\Tools\keyremap.exe --daemon)
      ```

### Command Line Arguments

```bash
USAGE:
    keyremap [OPTIONS]

OPTIONS:
    -c, --config <PATH>     Specify config file path, default is keyremap.toml in program directory
    -v, --verbose           Set log level, can be used multiple times for more detail, e.g., -v or -vv
        --logfile           Write logs to file (keyremap.log)
    -l, --listen           Listen mode, display all key events
        --dump             Display current config file content
    -d, --daemon           Background mode (Windows), hide console window
    -h, --help            Show help information
    -V, --version         Show version information

Examples:
# Start with specified config file
keyremap -c D:\my_config.toml

# Start in listen mode to view key events
keyremap --listen

# Start in debug mode to view detailed logs
keyremap -v

# Start in background mode
keyremap --daemon
```

### Configuration Example

```toml
version = "1.0"
name = "Config1"
comment = "Custom key mapping configuration"

[[key_mappings]]
name = "Pause to Insert"
comment = "Map Pause key to Insert key"
from.key = "Pause"
to.key = "Insert"

[[key_mappings]]
name = "Mouse side button to Ctrl+W"
from.button.Unknown = 2
to.combination = [
    { key = "ControlLeft" },
    { key = "KeyW" }
]
```

### Configuration File Description

- `version`: Configuration file version
- `name`: Configuration scheme name
- `comment`: Configuration description (optional)
- `key_mappings`: Key mapping list
  - `name`: Mapping name
  - `enable`: Whether enabled (optional, default is true)
  - `comment`: Mapping description (optional)
  - `from`: Source key
      - For keyboard key, format is `key = "keyValue"`
      - For mouse button, format is `button = "buttonValue"`
  - `to`: Target key or key combination
      - For single key, format is `key = "keyValue"`
      - For key combination, format is `combination = [ { key = "keyValue" }, ... ]`
  - Note: If key or button name is not standard, it will use Unknown number instead, format changes from `key = "keyValue"` to `key.Unknown = number`, same for button, refer to --listen output for specific values

## Development

### Build Project

```bash
cargo build --release
```

### Dependencies

- rdev: Keyboard and mouse event handling
- serde: Serialization support
- toml: Configuration file parsing
- log: Logging support
- env_logger: Logging environment configuration
- clap: Command line argument parsing
- windows-sys: Windows API support

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Issues and Pull Requests are welcome!

## FAQ

1. Q: Program won't start?  
   A: Please ensure you run the program with administrator privileges.

2. Q: Configuration changes don't take effect?  
   A: Please restart the program to load changes.

3. Q: How to temporarily disable a mapping?  
   A: Add `enable = false` to the corresponding mapping configuration.
