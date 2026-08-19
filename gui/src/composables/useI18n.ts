import { ref, computed } from 'vue';

export type Locale = 'zh' | 'en';

const LOCALE_STORAGE_KEY = 'diskdiff_locale';

const savedLocale = (localStorage.getItem(LOCALE_STORAGE_KEY) as Locale) || 'zh';
const currentLocale = ref<Locale>(savedLocale);

const messages: Record<Locale, Record<string, string>> = {
  zh: {
    // Navigation
    'nav.scan': '磁盘空间扫描',
    'nav.diff': '快照差异对比',
    'nav.snapshots': '快照历史管理',
    'nav.settings': '偏好设置',
    'nav.about': '关于软件',

    // TopBar & Common Actions
    'topbar.selectFolder': '选择要分析的目录...',
    'topbar.browse': '选择目录',
    'topbar.startScan': '开始扫描',
    'topbar.cancelScan': '停止扫描',
    'topbar.scanning': '正在飞速扫描磁盘...',
    'topbar.saveSnapshot': '保存快照',
    'topbar.searchPlaceholder': '过滤或搜索当前目录文件...',
    'topbar.viewTreemap': '股市热力图',
    'topbar.viewList': '详细列表',
    'topbar.back': '返回上一层',
    'topbar.root': '根目录',
    'topbar.totalSize': '总容量',
    'topbar.files': '文件数',
    'topbar.dirs': '目录数',

    // Treemap & List Views
    'treemap.empty': '当前目录下暂无文件或子目录',
    'treemap.drillDownHint': '双击目录方块可直接下钻展开，右键可显示操作菜单',
    'treemap.otherFiles': '其余微小文件汇总',
    'list.name': '名称',
    'list.size': '占用大小',
    'list.type': '类型',
    'list.filesCount': '子文件',
    'list.dirsCount': '子目录',
    'list.modified': '修改时间',
    'list.actions': '操作',
    'list.open': '浏览',
    'list.folder': '文件夹',
    'list.file': '文件',
    'list.reveal': '在访达/资源管理器中查看',

    // Snapshot Diff
    'diff.title': '磁盘快照差异对比',
    'diff.baseline': '基准快照 (旧)',
    'diff.target': '对比目标 (新)',
    'diff.currentActive': '⚡ 当前活动扫描 (内存常驻)',
    'diff.selectSnapshot': '选择历史快照...',
    'diff.compareNow': '立即执行多线程对比',
    'diff.comparing': '正在多线程并行对比与剪枝中...',
    'diff.deltaTotal': '总容量变动',
    'diff.added': '新增',
    'diff.removed': '已删除',
    'diff.modified': '已变动',
    'diff.unchanged': '无变化',
    'diff.treeTitle': '增量文件变化树',
    'diff.pruningHint': '已启用未修改子树并发剪枝加速',

    // Snapshot Manager
    'snapshots.title': '快照历史管理',
    'snapshots.subtitle': '采用 Zstd Level 9 极致压缩与 Bincode 序列化，百万级节点快照秒级加载',
    'snapshots.openExternal': '打开外部快照 (.snap)',
    'snapshots.emptyTitle': '暂无已保存的历史快照',
    'snapshots.emptyDesc': '在磁盘分析或扫描完成后，点击右上角「保存快照」即可在此归档管理。',
    'snapshots.snapSize': '快照文件大小',
    'snapshots.browse': '浏览此快照',
    'snapshots.loading': '载入中...',
    'snapshots.delete': '删除',
    'snapshots.deleteConfirm': '确定要删除这份快照吗？此操作无法撤销。',
    'snapshots.diffCurrent': '与当前扫描对比',
    'snapshots.storagePath': '当前快照存储目录',

    // Save Modal
    'saveModal.title': '保存磁盘快照',
    'saveModal.subtitle': '生成包含完整目录树的 .snap 高压缩二进制快照文件',
    'saveModal.nameLabel': '快照别名 / 备注',
    'saveModal.namePlaceholder': '请输入快照名称（如：系统清理前基准）...',
    'saveModal.storageLocation': '存储位置',
    'saveModal.saveBtn': '保存快照',
    'saveModal.saving': 'Zstd 压缩写入中...',
    'saveModal.cancelBtn': '取消',
    'saveModal.hint': '💡 采用 Zstd 9 级极速压缩，百万级目录通常仅需 2~10MB 存储空间。',

    // Settings
    'settings.title': '偏好设置',
    'settings.subtitle': '个性化配置语言、默认存储路径及热力图视觉偏好，所有配置自动持久化保存',
    'settings.language': '界面语言',
    'settings.languageDesc': '切换应用程序的界面显示语言，实时生效',
    'settings.storage': '默认快照存储位置',
    'settings.storageDesc': '保存生成的 .snap 快照文件以及快照历史管理所读取的本地目录',
    'settings.customPath': '自定义路径',
    'settings.defaultPath': '系统默认',
    'settings.changeDir': '更改目录',
    'settings.resetDefault': '恢复默认路径',
    'settings.openInFinder': '在文件管理器中打开',
    'settings.colorTheme': '热力图颜色主题',
    'settings.themeDesc': '选择您习惯的增减与容量视觉偏好',
    'settings.themeStockCn': '股市风格（红涨绿跌）',
    'settings.themeStockUs': '国际风格（绿涨红跌）',
    'settings.gainOccupied': '+ 涨 / 占用',
    'settings.lossFreed': '- 跌 / 释放',
    'settings.persistenceNotice': '✓ 设置项已自动保存在本地，下次启动软件依然有效。',
    'settings.successChangeDir': '已成功更新默认快照存储目录',
    'settings.successResetDir': '已恢复为系统默认存储目录',
    'settings.switchedZh': '已切换至简体中文',
    'settings.switchedEn': '已切换至英文',
    'settings.appliedStockCn': '已应用红涨绿跌配色',
    'settings.appliedStockUs': '已应用绿涨红跌配色',

    // About
    'about.title': '关于 DiskDiff',
    'about.version': '版本',
    'about.storyTitle': '写在前面',
    'about.story1': '开发 DiskDiff 的初衷非常简单：因为自己日常经常需要排查 Mac / Windows 电脑上到底是什么文件在悄悄蚕食磁盘，或者每次清理软件前后想知道到底删掉了什么、多出了什么。',
    'about.story2': '现有的工具要么界面老旧、要么扫描慢如蜗牛，更关键的是绝大部分工具都不支持快速保存快照并在日后进行「增量对比（Diff）」。',
    'about.story3': '于是我利用业余时间写了这个满足自己强迫症的小工具。底层用 Rust + Rayon 榨干多核多线程性能，前端用 Vue 3 打造直观现代的股市风格热力图，顺便把百万节点的单次通信内存控制在极致。',
    'about.featuresTitle': '核心特性',
    'about.feature1': '🚀 多线程并发扫描：Work-Stealing 线程池，极速遍历数百万文件与目录',
    'about.feature2': '📊 股市风格热力图：红绿色彩清晰展现目录空间占比与增减趋势',
    'about.feature3': '⚡ 增量对比与剪枝：对比任意两份快照，未修改子树快速剪枝跳过',
    'about.feature4': '💾 极高压缩比快照：Zstd Level 9 + Bincode，百万节点仅占几兆，即存即取',
    'about.techTitle': '技术架构',

    // Common
    'common.loading': '加载中...',
    'common.confirm': '确认',
    'common.cancel': '取消',
  },

  en: {
    // Navigation
    'nav.scan': 'Disk Space Scan',
    'nav.diff': 'Snapshot Diff',
    'nav.snapshots': 'Snapshots',
    'nav.settings': 'Settings',
    'nav.about': 'About',

    // TopBar & Common Actions
    'topbar.selectFolder': 'Select directory to analyze...',
    'topbar.browse': 'Browse',
    'topbar.startScan': 'Start Scan',
    'topbar.cancelScan': 'Stop Scan',
    'topbar.scanning': 'Scanning disk at high speed...',
    'topbar.saveSnapshot': 'Save Snapshot',
    'topbar.searchPlaceholder': 'Filter or search current directory files...',
    'topbar.viewTreemap': 'Treemap',
    'topbar.viewList': 'List View',
    'topbar.back': 'Back',
    'topbar.root': 'Root',
    'topbar.totalSize': 'Total Size',
    'topbar.files': 'Files',
    'topbar.dirs': 'Folders',

    // Treemap & List Views
    'treemap.empty': 'No files or folders in current directory',
    'treemap.drillDownHint': 'Double click folder to drill down, right click for options',
    'treemap.otherFiles': 'Other smaller items',
    'list.name': 'Name',
    'list.size': 'Size',
    'list.type': 'Type',
    'list.filesCount': 'Files',
    'list.dirsCount': 'Folders',
    'list.modified': 'Modified',
    'list.actions': 'Actions',
    'list.open': 'Open',
    'list.folder': 'Folder',
    'list.file': 'File',
    'list.reveal': 'Reveal in Finder / Explorer',

    // Snapshot Diff
    'diff.title': 'Snapshot Differential Analysis',
    'diff.baseline': 'Baseline Snapshot (Old)',
    'diff.target': 'Target Snapshot (New)',
    'diff.currentActive': '⚡ Active Scan in Memory',
    'diff.selectSnapshot': 'Select a snapshot...',
    'diff.compareNow': 'Start Differential Analysis',
    'diff.comparing': 'Diffing snapshots & pruning unchanged subtrees...',
    'diff.deltaTotal': 'Total Delta',
    'diff.added': 'Added',
    'diff.removed': 'Removed',
    'diff.modified': 'Modified',
    'diff.unchanged': 'Unchanged',
    'diff.treeTitle': 'Delta File Hierarchy',
    'diff.pruningHint': 'Concurrent unchanged subtree pruning enabled',

    // Snapshot Manager
    'snapshots.title': 'Snapshot Manager',
    'snapshots.subtitle': 'Zstd Level 9 compression & Bincode serialization, loads million-node trees in seconds',
    'snapshots.openExternal': 'Open External Snapshot (.snap)',
    'snapshots.emptyTitle': 'No Saved Snapshots Yet',
    'snapshots.emptyDesc': 'After scanning a directory, click "Save Snapshot" in the top bar to archive it here.',
    'snapshots.snapSize': 'Snapshot Size',
    'snapshots.browse': 'Browse Snapshot',
    'snapshots.loading': 'Loading...',
    'snapshots.delete': 'Delete',
    'snapshots.deleteConfirm': 'Are you sure you want to delete this snapshot? This cannot be undone.',
    'snapshots.diffCurrent': 'Compare with Active Scan',
    'snapshots.storagePath': 'Snapshot Storage Directory',

    // Save Modal
    'saveModal.title': 'Save Disk Snapshot',
    'saveModal.subtitle': 'Create a high-compression .snap binary snapshot of the directory tree',
    'saveModal.nameLabel': 'Snapshot Name / Label',
    'saveModal.namePlaceholder': 'e.g., Before System Cleanup Baseline...',
    'saveModal.storageLocation': 'Save Location',
    'saveModal.saveBtn': 'Save Snapshot',
    'saveModal.saving': 'Compressing & Writing with Zstd...',
    'saveModal.cancelBtn': 'Cancel',
    'saveModal.hint': '💡 Using Zstd level 9 compression, millions of nodes usually take only 2~10MB.',

    // Settings
    'settings.title': 'Preferences & Settings',
    'settings.subtitle': 'Customize language, default snapshot storage directory, and visualization palettes',
    'settings.language': 'Interface Language',
    'settings.languageDesc': 'Switch application interface display language instantly',
    'settings.storage': 'Default Snapshot Storage Location',
    'settings.storageDesc': 'Directory where generated .snap snapshot files are saved and loaded',
    'settings.customPath': 'Custom Directory',
    'settings.defaultPath': 'Default Directory',
    'settings.changeDir': 'Change Directory',
    'settings.resetDefault': 'Reset to Default',
    'settings.openInFinder': 'Open in File Manager',
    'settings.colorTheme': 'Heatmap Color Theme',
    'settings.themeDesc': 'Choose your preferred color theme for disk capacity changes',
    'settings.themeStockCn': 'CN Stock Style (Red up / Green down)',
    'settings.themeStockUs': 'International Style (Green up / Red down)',
    'settings.gainOccupied': '+ Gain / Occupied',
    'settings.lossFreed': '- Loss / Freed',
    'settings.persistenceNotice': '✓ All settings are automatically saved locally and persisted across app restarts.',
    'settings.successChangeDir': 'Successfully updated snapshot storage directory',
    'settings.successResetDir': 'Reset to system default snapshot directory',
    'settings.switchedZh': 'Switched to Simplified Chinese',
    'settings.switchedEn': 'Switched to English',
    'settings.appliedStockCn': 'Applied CN stock color theme',
    'settings.appliedStockUs': 'Applied International color theme',

    // About
    'about.title': 'About DiskDiff',
    'about.version': 'Version',
    'about.storyTitle': 'Behind the Project',
    'about.story1': 'The motivation for building DiskDiff was simple: I often needed to inspect what was silently eating up disk space on my Mac and Windows machines, or compare exact differences before and after cleaning up folders.',
    'about.story2': 'Existing tools were either sluggish, outdated, or completely lacked snapshot diff capabilities to compare directory states over time.',
    'about.story3': 'So I built DiskDiff in my spare time: Rust + Rayon for raw multi-threaded throughput, Vue 3 for a modern stock-market treemap, and an ultra-lean IPC architecture.',
    'about.featuresTitle': 'Key Features',
    'about.feature1': '🚀 Multi-Threaded Scanning: Work-stealing Rayon pool traverses millions of files in seconds',
    'about.feature2': '📊 Treemap Visualization: Color-coded heatmaps clearly display disk occupancy & trends',
    'about.feature3': '⚡ Subtree Pruned Diff: Compare any two snapshots with multi-threaded pruning acceleration',
    'about.feature4': '💾 High-Compression Snapshots: Zstd 9 + Bincode takes only a few MBs for millions of nodes',
    'about.techTitle': 'Tech Architecture',

    // Common
    'common.loading': 'Loading...',
    'common.confirm': 'Confirm',
    'common.cancel': 'Cancel',
  },
};

export function useI18n() {
  const t = (key: string, defaultVal?: string): string => {
    const localeDict = messages[currentLocale.value] || messages.zh;
    return localeDict[key] || defaultVal || key;
  };

  const setLocale = (locale: Locale) => {
    currentLocale.value = locale;
    localStorage.setItem(LOCALE_STORAGE_KEY, locale);
  };

  const isZh = computed(() => currentLocale.value === 'zh');
  const isEn = computed(() => currentLocale.value === 'en');

  return {
    t,
    locale: currentLocale,
    setLocale,
    isZh,
    isEn,
  };
}
