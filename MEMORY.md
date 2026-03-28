# Voxis 项目记忆

## 基本信息
- 项目名称: Voxis / Voix
- GitHub: https://github.com/winsonwq/voxis
- 架构: Tauri 2.x + React + TypeScript
- 目标: 本地优先的 AI 语音输入 → 自动润色 → 文本注入

## 当前状态 (2026-03-28)

### 已完成
- 项目初始化 (Tauri 2.x + React)
- Whisper.cpp 本地转写 (whisper-rs)
- LLM 润色 (Ollama / OpenAI)
- 文本注入 (enigo)
- SQLite 持久化 (设置 + 历史)
- 基本 UI (录音状态、音频电平条、历史记录)
- 热键监听框架 (hotkey.rs 已创建)
- 录音启停框架 (audio.rs 已重写)
- Rust 编译 check 通过 (代码框架 OK)

### 未完成 (P0)
1. **全局热键** — hotkey.rs 已建但未与主循环集成
2. **录音启停** — audio start/stop 框架有了，但 toggle_recording 流程需联调
3. **Accessibility 权限** — macOS 下 enigo 需要无障碍权限，没检查也没引导

### 未完成 (P1)
4. 设置面板 UI + 后端 (open_settings 命令未实现)
5. 录音时波形/音量可视化 (UI 有 level-bar，数据流通了)
6. 托盘图标状态更新

### 未完成 (P2)
7. 注入失败回退剪贴板
8. 通知 (tauri-plugin-notification 引入了但没用)

## 技术栈
- Tauri 2.x (窗口 + 托盘)
- React + TypeScript (前端)
- whisper-rs 0.16 (本地 STT)
- cpal (麦克风录音)
- enigo (文本注入)
- rusqlite (SQLite 持久化)
- reqwest (HTTP LLM 调用)
- tauri-plugin-global-shortcut (全局快捷键)

## 开发环境
- Rust 1.94.1 (via rustup)
- Node.js 22.22.0 (via nvm)
- 构建: `npm run tauri dev` 或 `npm run tauri build`
- 编译检查: `cargo check` (在 src-tauri/ 目录)

## 相关参考
- nanowhisper (https://github.com/jicaiinc/nanowhisper) — 类似项目，热键和录音流程参考

## Ralph Loop
- Cron: 每天 00:00 自动触发
- 脚本: scripts/ralph-loop.sh
- 状态: memory/ralph-loop-state.json
- 报告: memory/ralph-loop-report-YYYY-MM-DD.md
