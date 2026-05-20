# ZIP 工具拖入部署 — 设计文档

## 目标

在设置页的 Tools 区，支持用户拖入 CI 构建的 CLI 工具 ZIP 包，自动解压到 host exe 所在目录，并自动回填前端输入框 + 后端优先使用设置路径。

## 设计

### 1. 后端新命令 `extract_tools_zip`

**文件**: `host/src-tauri/src/commands.rs`

```
extract_tools_zip(zip_path: String) -> HashMap<String, String>
```

- 使用 `zip` crate 读取 ZIP
- 解压到 host exe 所在目录（`std::env::current_exe().parent()`）
- 覆盖同名文件（用户需要主动拖入，视为有意覆盖）
- 扫描解压后的文件，匹配已知工具名:
  - `audiobook-scanner` / `audiobook-scanner.exe`
  - `audiobook-splitter` / `audiobook-splitter.exe`
  - `audiobook-transcriber` / `audiobook-transcriber.exe`
  - `audiobook-organizer` / `audiobook-organizer.exe`
- 返回工具名→完整路径的 HashMap

**Cargo.toml 新增依赖**: `zip = "2"`

### 2. 前端拖拽区域

**文件**: `host/src/components/Settings.tsx`

- 在 Tools 区域顶部添加一个 `.zip-dropper` 拖拽框（虚线边框，拖入高亮）
- 使用 Tauri 的 `onDragDropEvent` 或 HTML5 Drag & Drop
- 只接受 `.zip` 文件
- 拖入后:
  1. 调用 `invoke('extract_tools_zip', { zipPath })`
  2. 用返回的 HashMap 更新 `SettingsContext` 中 4 个工具路径
  3. `localStorage` 自动持久化

**样式**: 与现有暗色主题一致，拖入时边框变亮 + "提取中..." 状态

### 3. 后端 `process.rs` 改造

**文件**: `host/src-tauri/src/process.rs`

```rust
fn find_binary(name: &str, tool_paths: &HashMap<String, String>) -> String {
    // 1. 优先查 tool_paths
    if let Some(path) = tool_paths.get(name) {
        if !path.is_empty() { return path.clone(); }
    }
    // 2. 原逻辑：查 exe 同级 + release 目录
    // 3. fallback PATH
}
```

所有 `spawn_*` 函数签名增加 `tool_paths: &HashMap<String, String>` 参数。

### 4. AppState 新增 `tool_paths`

**文件**: `host/src-tauri/src/state.rs`

```rust
pub struct AppState {
    // ... 现有字段
    pub tool_paths: Mutex<HashMap<String, String>>,
}
```

新增命令 `set_tool_paths(paths: HashMap<String, String>)` 让前端同步设置到后端 state。

新增命令 `get_tool_paths() -> HashMap<String, String>` 让前端启动时恢复后端 state。

### 5. 前后端同步流程

| 步骤 | 动作 |
|------|------|
| App 启动 | 前端从 `localStorage` 读路径 → `invoke('set_tool_paths')` → 后端保存 |
| 拖入 ZIP | 前端 `invoke('extract_tools_zip')` → 解压 + 回填 UI + `set_tool_paths` |
| 手动改路径 | 前端 onChange → 更新 `localStorage` + `invoke('set_tool_paths')` |
| 执行功能 | `commands.rs` 从 `AppState.tool_paths` 读 → 传给 `process.rs` |

## 受影响的文件

| 文件 | 改动 |
|------|------|
| `host/src-tauri/Cargo.toml` | 加 `zip = "2"` |
| `host/src-tauri/src/commands.rs` | 加 `extract_tools_zip`, `set_tool_paths`, `get_tool_paths`；修改现有命令传 `tool_paths` |
| `host/src-tauri/src/process.rs` | `find_binary` + `spawn_*` 加 `tool_paths` 参数 |
| `host/src-tauri/src/state.rs` | `AppState` 加 `tool_paths` 字段 |
| `host/src-tauri/src/lib.rs` | 注册新命令；初始化 `tool_paths` |
| `host/src/components/Settings.tsx` | 加拖拽框 + 解压回填逻辑 |
| `host/src/SettingsContext.tsx` | 加 `setToolPaths` 方法；启动时同步到后端 |

## 不做的事

- 不添加进度条（ZIP 通常很小，解压瞬间完成）
- 不改动现有 `find_binary` fallback 逻辑
- 不校验 ZIP 内二进制是否有效（由 CI 保证质量）
