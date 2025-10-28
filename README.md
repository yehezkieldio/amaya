# Amaya

**⚠️ Note: This project is no longer being actively maintained.**

## Overview

Amaya is a command-line tool designed to automate opinionated development configurations for your projects. It provides a flexible provider-based system that allows you to install, manage, and remove development tool configurations with ease.

## Features

- **Configuration Management**: Install and remove development tool configurations via a simple CLI
- **Provider System**: Extensible JSON-based provider system for defining configurations
- **Package Manager Support**: Currently supports Bun package manager
- **Interactive Selection**: Choose configurations interactively or specify them directly
- **Script Automation**: Automatically add scripts to your package.json
- **VS Code Integration**: Built-in support for VS Code settings management
- **Health Checks**: Run diagnostics to verify prerequisites for configurations
- **Initial Setup**: Bootstrap your configuration directory with sensible defaults

## Getting Started

### Prerequisites

- Rust toolchain (1.70 or later)
- Cargo package manager

### Building from Source

1. Clone the repository:
```bash
git clone https://github.com/yehezkieldio/amaya.git
cd amaya
```

2. Build the project:
```bash
cargo build --release
```

3. The compiled binary will be available at `target/release/amaya`

4. (Optional) Install globally:
```bash
cargo install --path .
```

### Usage

Initialize the configuration directory:
```bash
amaya init
```

List available configurations:
```bash
amaya list
```

Install a configuration (interactive):
```bash
amaya install
```

Install a specific configuration:
```bash
amaya install --config biome
```

Remove a configuration:
```bash
amaya remove --config biome
```

Check system prerequisites:
```bash
amaya doctor
```

## License

MIT License

Copyright (c) 2024 Yehezkiel Dio Sinolungan

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
