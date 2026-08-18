<div align="center">

# 💽 DiskDiff

**轻量、极速的磁盘空间分析与快照增量对比工具**

[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-24C8D5.svg?style=flat-square&logo=tauri)](https://tauri.app/)
[![Vue 3](https://img.shields.io/badge/Vue-3.5%2B-4FC08D.svg?style=flat-square&logo=vue.js)](https://vuejs.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey.svg?style=flat-square)](#-跨平台支持)

</div>

---

## 💡 为什么写这个软件？

平时电脑用着用着磁盘空间经常告急。虽然市面上有很多优秀的磁盘清理和分析软件，但它们大多只能看**「当前哪个文件夹最大」**，却很难回答**「从上周到今天，到底是什么东西在偷偷暴涨」**。

传统的比对工具要么体积庞大臃肿，要么不支持直观的热力图下钻定位。

于是干脆用 **Rust + Tauri 2.0 + Vue 3** 为自己写了这个轻量、纯粹的小工具：**快速给文件夹拍个快照，隔段时间拉出来比对，哪里涨了、哪里删了以股市热力图直接呈现，一目了然！**

---

## 🚀 核心功能特性

### 1. ⚡ Rayon 多核并发磁盘扫描
- **多核心工作窃取**：利用 Rust Rayon 工作窃取线程池（Work-Stealing Pool）并行并发遍历文件树，跑满现代 NVMe SSD 的极致 IOPS。
- **自底向上逐层归集**：自动统计每层目录的总体积、文件数与子目录数，大目录永远排在最前。
- **毫秒级防抖流式反馈**：每 50ms 节流向前端推送原子扫描进度，丝滑流畅绝不阻塞 UI。
- **软硬链接安全防护**：自动识别 Unix Inode 防止硬链接重复计重，不跟随软链接防止循环死锁。

### 2. 📈 股市风格容量热力图 (Treemap)
- **单层下钻架构**：基于 D3.js 自定义实现的 Squarified Treemap，仅渲染当前层级（Top-N 聚类合并），轻松承载数百万节点而不卡顿。
- **红绿/绿红涨跌双配色**：支持 A 股红涨绿跌（红增绿减）与国际绿涨红跌双配色模式切换。
- **多视图切换**：支持热力图与列表视图无缝切换，右键可快速定位系统文件管理器（Finder / Explorer）。

### 3. 🔍 深度快照增量比对 (Diff Engine)
- **同级目录智能剪枝（Subtree Pruning）**：对比两期快照时，若检测到子目录元数据一致，瞬间剪枝跳过数十万未修改节点，比对速度成倍提升。
- **双路并发解压**：通过 `rayon::join` 双线程并行解压和反序列化两份历史快照，载入耗时直接减半。
- **多维度变动计算**：自动精准计算每个节点的绝对变动体积（$\Delta \text{size}$）与相对涨跌百分比（$\Delta \%$）。

### 4. 🗜️ 自定义二进制高压缩快照格式 (`.snap`)
- **独立元数据头部**：专有二进制容器设计，毫秒级快速读取快照元数据（名称、路径、容量、文件数），无需解压整颗多叉树。
- **Zstd Level 9 + Bincode**：数百万节点的数据树压缩后仅占数十兆，比纯文本节省 90% 以上存储空间。

### 5. 🧠 后端常驻与极简前端内存
- 百万级完整数据树常驻 Rust 后端内存；
- 前端通过 Tauri IPC 仅按需拉取单层数据（单次传输 < 10KB），前端内存占用稳定在 **< 5MB**，彻底告别 Webview 内存膨胀卡死。

---

## 🖥️ 跨平台支持

DiskDiff 原生支持全主流操作系统与架构：

- **macOS**：Apple Silicon (M1/M2/M3/M4, `arm64`)
- **Windows**：Windows 10 / 11 (`x86_64`)
- **Linux**：Ubuntu / Debian / Arch (`x86_64` & `arm64`)

---

## 🛠️ 本地开发与构建

### 1. 环境准备
- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://www.rust-lang.org/) (v1.80+)
- [Tauri 2.0 CLI](https://v2.tauri.app/start/prerequisites/)

### 2. 安装依赖
```bash
npm install
```

### 3. 启动本地开发
```bash
npm run tauri dev
```

### 4. 本地打包构建
```bash
npm run tauri build
```
打包输出路径位于 `src-tauri/target/release/bundle/`。

---

## 📄 开源许可证

本项目基于 [MIT License](LICENSE) 协议开源。
