# Linkron Note 构建和部署文档

## 目录
- [GitHub Actions 自动构建](#github-actions-自动构建)
- [自动更新配置](#自动更新配置)
- [GitHub Secrets 配置](#github-secrets-配置)
- [发布流程](#发布流程)
- [手动构建](#手动构建)

---

## GitHub Actions 自动构建

### 工作流程说明

项目使用 GitHub Actions 自动构建多平台应用：

- **macOS (ARM64)**: Apple Silicon 芯片
- **macOS (x86_64)**: Intel 芯片
- **Linux**: AppImage 和 deb 包
- **Windows**: MSI 和 NSIS 安装包

### 触发条件

工作流会在以下情况下触发：

1. **推送到主分支**: `main` 或 `dev`
2. **创建版本标签**: 以 `v` 开头的标签（如 `v1.0.0`）
3. **Pull Request**: 到 `main` 或 `dev`
4. **手动触发**: 在 GitHub Actions 页面手动运行

---

## GitHub Secrets 配置

### 必需的 Secrets

在 GitHub 仓库设置中添加以下 Secrets（Settings → Secrets and variables → Actions）：

| Secret 名称 | 说明 | 获取方式 |
|------------|------|---------|
| `TAURI_PRIVATE_KEY` | Tauri 签名私钥 | 从本地密钥文件获取 |
| `TAURI_KEY_PASSWORD` | 私钥密码 | 设置的密码（如：zw123456） |
| `GITHUB_TOKEN` | GitHub Token | 自动提供，无需配置 |

### 获取私钥内容

在项目根目录下，私钥文件位于 `.tauri-keys`，你需要：

1. **读取私钥内容**（不包含注释行）：
   ```bash
   cat .tauri-keys
   ```

2. **复制私钥**（通常是 `-----BEGIN TAURI PRIVATE KEY-----` 之后到 `-----END TAURI PRIVATE KEY-----` 之间的内容，或整个文件内容）

3. **添加到 GitHub Secrets**：
   - 进入仓库 Settings → Secrets and variables → Actions
   - 点击 "New repository secret"
   - Name: `TAURI_PRIVATE_KEY`
   - Value: 粘贴私钥内容

⚠️ **注意**：私钥非常重要，请勿泄露或提交到 Git！

---

## 发布流程

### 创建新版本发布

1. **更新版本号**

   编辑 `src-tauri/tauri.conf.json` 和 `package.json`，更新版本号：
   ```json
   "version": "0.2.0"
   ```

2. **创建并推送标签**
   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

3. **自动构建和发布**

   - GitHub Actions 会自动开始构建
   - 构建完成后，自动创建 GitHub Release
   - 所有平台的安装包和更新文件会上传到 Release

4. **自动更新生效**

   应用会自动检测新版本并提示用户更新

---

## 自动更新配置

### 更新端点

应用配置从以下地址检查更新：
```
https://github.com/ittking/linkron-note/releases/latest/download/latest.json
```

### 更新文件结构

每次 Release 会包含以下文件：

```
Release v0.2.0/
├── linkron_0.2.0_amd64.AppImage      # Linux AppImage
├── linkron_0.2.0_amd64.deb           # Linux deb
├── linkron_0.2.0_x64.app.tar.gz      # macOS ARM64
├── linkron_0.2.0_x64.dmg             # macOS ARM64
├── linkron_0.2.0_x86_64.app.tar.gz   # macOS x86_64
├── linkron_0.2.0_x86_64.dmg          # macOS x86_64
├── linkron_0.2.0_x64-setup.exe       # Windows NSIS
├── linkron_0.2.0_x64.msi             # Windows MSI
├── latest-linux.json                 # Linux 更新配置
├── latest-macos-arm64.json           # macOS ARM64 更新配置
├── latest-macos-x86_64.json          # macOS x86_64 更新配置
└── latest-windows.json               # Windows 更新配置
```

### 用户更新流程

1. 应用启动时自动检查更新
2. 发现新版本时显示更新提示
3. 用户确认后下载更新
4. 下载完成后自动安装并重启应用

---

## 手动构建

### 本地构建（无需签名）

```bash
# 安装依赖
pnpm install

# 构建前端
pnpm build

# 构建 Tauri 应用
pnpm tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`

### 本地构建（带签名）

```bash
# 设置环境变量
export TAURI_SIGNING_PRIVATE_KEY=.tauri-keys
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=zw123456

# 构建
pnpm tauri build
```

---

## 故障排除

### 构建失败

1. **权限错误**: 确保 GitHub Secrets 配置正确
2. **版本号不一致**: 确保 `tauri.conf.json` 和 `package.json` 版本一致
3. **签名失败**: 检查私钥格式和密码是否正确

### 更新不工作

1. **检查更新端点**: 确保能访问 `latest.json`
2. **版本号递增**: 新版本号必须大于当前版本
3. **Release 文件完整**: 确保所有必需的文件都已上传

---

## 最佳实践

1. **版本管理**
   - 使用语义化版本 (Semantic Versioning): `主版本.次版本.修订版`
   - 主版本: 不兼容的 API 变更
   - 次版本: 向下兼容的功能新增
   - 修订版: 向下兼容的问题修复

2. **发布前检查**
   - [ ] 更新版本号
   - [ ] 测试所有平台
   - [ ] 更新 CHANGELOG
   - [ ] 确保私钥安全

3. **密钥安全**
   - 定期备份私钥文件
   - 不要在任何地方公开私钥
   - 使用强密码保护私钥

---

## 相关链接

- [Tauri 官方文档](https://tauri.app/)
- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [项目仓库](https://github.com/ittking/linkron-note)
