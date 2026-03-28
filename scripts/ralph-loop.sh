#!/bin/bash
# Ralph Loop trigger script
# Usage: ./ralph-loop.sh
# Runs the Ralph Loop autonomous development cycle in the Voxis project

PROJECT_DIR="$HOME/.openclaw/workspace/voxis"
LOG_FILE="$PROJECT_DIR/memory/ralph-loop.log"
STATE_FILE="$PROJECT_DIR/memory/ralph-loop-state.json"
DATE=$(date +%Y-%m-%d)
TIMESTAMP=$(date +"%Y-%m-%dT%H:%M:%S%z")
OPENCLAW="$HOME/.nvm/versions/node/v22.22.0/bin/openclaw"

mkdir -p "$PROJECT_DIR/memory"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

cd "$PROJECT_DIR"

# Check git status
if [[ -n $(git status --porcelain) ]]; then
    log "WARNING: uncommitted changes, stashing before Ralph Loop"
    git stash push -m "ralph-loop-autosave-$(date +%s)" || true
fi

# Check current branch
BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "main")
log "Starting Ralph Loop on branch: $BRANCH"

# Update state
cat > "$STATE_FILE" << EOF
{
  "lastRun": "$TIMESTAMP",
  "lastPhase": "SURVEY",
  "branch": "$BRANCH",
  "completedThisSession": []
}
EOF

log "Invoking OpenClaw Ralph Loop agent..."

# Run the Ralph Loop via OpenClaw agent
# Using a dedicated session for Voxis work
$OPENCLAW agent \
    --session-id voxis-ralph-loop \
    --message "启动 Ralph Loop。

项目目录: $PROJECT_DIR

请执行以下步骤：

1. 读取 $PROJECT_DIR/AGENTS.md 中的 RALPH LOOP 章节，理解完整流程
2. 读取 $PROJECT_DIR/memory/ralph-loop-state.json 了解当前状态
3. 从 SURVEY 阶段开始：读取 REQUIREMENT.md、查看 git log、检查现有代码实现
4. 确定最高优先级的未完成功能（P0 优先：全局热键、录音启停流程、Accessibility 权限）
5. 在 feat/ 分支上实现
6. 运行 cargo check 验证编译
7. 完成后：
   - git commit
   - 更新 memory/ralph-loop-state.json 的 lastPhase 为 DONE
   - 将本次完成情况写入 $PROJECT_DIR/memory/ralph-loop-report-$DATE.md

报告格式：
## Ralph Loop Report $DATE

### Survey Findings
发现的问题...

### Implemented
完成的功能...

### Test Results
编译/测试结果...

### Next Steps
下一步..." 2>&1 | tee -a "$LOG_FILE"

EXIT_CODE=$?

if [[ $EXIT_CODE -eq 0 ]]; then
    log "Ralph Loop completed successfully"
else
    log "Ralph Loop exited with code $EXIT_CODE - check logs"
fi
