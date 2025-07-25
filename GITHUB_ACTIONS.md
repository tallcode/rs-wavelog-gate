# GitHub Actions 构建配置说明

## 修改概述

已修改 `.github/workflows/release.yml` 以支持带图标的跨平台构建：

### 主要改动

#### 1. **macOS构建改进**
```yaml
- name: Install cargo-bundle (macOS)
  if: matrix.os == 'macos-latest'
  run: cargo install cargo-bundle

- name: Create macOS app bundle with icon
  if: matrix.os == 'macos-latest'  
  run: cargo bundle --release
```

- 自动安装 `cargo-bundle` 工具
- 生成带图标的 `.app` bundle 而不是普通可执行文件
- 上传完整的 app bundle

#### 2. **Windows构建**
- 保持原有构建流程
- 图标通过 `build.rs` 自动嵌入到 `.exe` 文件中
- 无需额外配置

#### 3. **发布优化**
```yaml
- name: Prepare macOS app bundle
  run: |
    cd rs-wavelog-gate-macos
    zip -r ../rs-wavelog-gate-macos.zip "Wavelog Gate.app"

- name: Prepare Windows executable  
  run: |
    cd rs-wavelog-gate-windows
    chmod +x rs-wavelog-gate.exe
    zip ../rs-wavelog-gate-windows.zip rs-wavelog-gate.exe
```

- macOS：打包整个 `.app` bundle
- Windows：直接打包 `.exe` 文件
- 使用 ZIP 格式而不是 tar.gz（更适合应用分发）

## 构建产物

### macOS
- **文件**: `rs-wavelog-gate-macos.zip`
- **内容**: 完整的 `Wavelog Gate.app` bundle
- **图标**: 包含 `icon.icns`，在 Finder 和 Dock 中显示
- **使用**: 解压后拖到应用程序文件夹即可

### Windows  
- **文件**: `rs-wavelog-gate-windows.zip`
- **内容**: `rs-wavelog-gate.exe` 可执行文件
- **图标**: 嵌入的 `icon.ico`，在资源管理器和任务栏中显示
- **使用**: 解压后直接运行

## 触发发布

当推送版本标签时自动触发：

```bash
# 创建并推送标签
git tag v1.0.0
git push origin v1.0.0
```

## 本地测试

使用 `test-actions-build.sh` 脚本在本地测试GitHub Actions的构建逻辑：

```bash
./test-actions-build.sh
```

该脚本会：
1. 清理之前的构建
2. 执行与GitHub Actions相同的构建步骤
3. 生成相同格式的输出文件
4. 验证图标是否正确包含

## 验证图标

### macOS
```bash
# 解压后检查app bundle
unzip rs-wavelog-gate-macos.zip
open "Wavelog Gate.app"  # 应显示图标
```

### Windows
```bash
# 解压后检查exe文件
unzip rs-wavelog-gate-windows.zip
# 在文件资源管理器中查看应显示图标
```

## 故障排除

如果构建失败：

1. **cargo-bundle 安装失败**
   - 检查网络连接
   - GitHub Actions会自动重试

2. **图标文件不存在**
   - 确保 `icon.icns`, `icon.ico`, `icon.png` 在仓库根目录
   - 检查文件大小和格式

3. **bundle创建失败**
   - 检查 `Cargo.toml` 中的 `[package.metadata.bundle]` 配置
   - 确保图标文件路径正确
