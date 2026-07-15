export type Language = 'zh_CN' | 'en' | 'ru';

export const LANGUAGE_DISPLAY: Record<Language, string> = {
  zh_CN: '简体中文',
  en: 'English',
  ru: 'Русский',
};

export function systemLocaleToLanguage(): Language {
  const locale = (typeof navigator !== 'undefined' && navigator.language) || 'en-US';
  const lower = locale.toLowerCase().replace('_', '-');
  if (lower.startsWith('zh')) return 'zh_CN';
  if (lower.startsWith('ru')) return 'ru';
  return 'en';
}

type TranslationMap = Record<string, string>;

const translations: Record<Language, TranslationMap> = {
  zh_CN: {
    'menu.home': '首页', 'menu.settings': '设置', 'menu.about': '关于',
    'app.title_brand': '澪刻 Blitz Installer', 'app.title_sub': 'Tanks Blitz 本地化安装器',
    'button.refresh': '刷新', 'button.auto_scan': '自动扫描', 'button.install': '安装', 'button.install_localization': '安装本地化',
    'button.uninstall': '卸载', 'button.uninstall_localization': '卸载本地化', 'button.launch': '启动',
    'button.settings': '设置', 'button.update': '更新', 'button.force_kill': '强制关闭',
    'button.save': '保存', 'button.cancel': '取消', 'button.browse': '浏览', 'button.confirm': '确认',
    'button.select_language': '选择语言',
    'status.localization_not_installed': '未安装本地化', 'status.not_installed': '未安装', 'status.localization_installed': '已安装本地化',
    'status.no_compatible_version': '暂无支持的版本', 'status.up_to_date': '已是最新', 'status.needs_update': '需要更新',
    'status.running': '运行中', 'status.not_running': '未运行', 'status.scanning': '正在扫描...',
    'status.no_instances': '未检测到 Tanks Blitz 实例', 'status.waiting_close': '等待游戏关闭...',
    'progress.preparing': '准备下载', 'progress.l10n': '正在下载本地化包', 'progress.font': '正在下载字体包',
    'progress.extracting': '正在解压', 'progress.extracted': '解压完成', 'progress.installing_l10n': '正在安装本地化包',
    'progress.installing_font': '正在安装字体包', 'progress.done': '安装完成',
    'label.scan_added': '新增 {count} 个实例', 'label.scan_no_new': '未发现新实例',
    'dialog.install_failed': '安装失败', 'dialog.install_failed_detail': '安装失败：{error}',
    'dialog.uninstall_failed': '卸载失败', 'dialog.uninstall_success': '卸载成功', 'dialog.uninstall_failed_detail': '卸载失败：{error}',
    'dialog.settings_saved': '设置已保存', 'dialog.kill_confirm': '游戏正在运行，需要关闭后才能继续。是否强制关闭？',
    'dialog.not_implemented': '此功能尚未实现', 'dialog.not_implemented_detail': '下载与安装功能将在后续版本中开放。',
    'label.game': '游戏', 'label.scanning': '扫描中...', 'label.blitz_live': 'Tanks Blitz 正式服',
    'label.path': '路径', 'label.version': '版本', 'label.localization': '本地化', 'label.font': '字体包', 'label.language': '语言',
    'settings.language': '语言', 'settings.theme': '主题', 'settings.proxy': '代理设置',
    'settings.use_system_proxy': '使用系统代理', 'settings.custom_proxy': '自定义代理',
    'settings.host': '主机', 'settings.port': '端口', 'settings.username': '用户名', 'settings.password': '密码',
    'settings.auth_optional': '认证信息 (可选)', 'settings.proxy_host_required': '主机地址不能为空', 'settings.proxy_port_required': '端口号不能为空',
    'settings.files': '文件', 'settings.cache': '缓存', 'settings.data_dir_label': '数据目录', 'settings.open_dir': '打开目录',
    'settings.clear_cache': '清除缓存', 'settings.clear_cache_confirm_title': '确认清除缓存',
    'settings.clear_cache_confirm_msg': '下载缓存能够有效提升安装效率。但如果您安装的本地化包一直无法被应用正常读取，请尝试该选项。确认清除下载缓存？',
    'settings.clear_cache_success': '缓存已清除', 'label.latest': '最新',
    'about.version': '版本', 'about.check_update': '检查更新', 'about.checking_update': '正在检查更新…',
    'about.up_to_date': '已是最新版本', 'about.update_available': '发现新版本 v{version}', 'about.update_now': '立即更新',
    'about.update_download_progress': '更新包下载中… {percent}%', 'about.website': '官方网站', 'about.github': 'GitHub 仓库',
    'about.license': '开源许可', 'about.copyright': '© 2026 LocalizedBlitz',
    'error.update_check_failed': '检查更新失败', 'error.update_download_failed': '下载更新失败',
    'error.scan_failed': '扫描失败', 'error.launch_failed': '启动失败', 'error.kill_failed': '强制关闭失败',
    'instance.path': '路径', 'instance.version': '游戏版本', 'instance.type': '类型', 'instance.actions': '操作', 'instance.browse_manual': '手动浏览',
  },
  en: {
    'menu.home': 'Home', 'menu.settings': 'Settings', 'menu.about': 'About',
    'app.title_brand': 'Blitz Installer', 'app.title_sub': 'Localization installer for Tanks Blitz',
    'button.refresh': 'Refresh', 'button.auto_scan': 'Auto Scan', 'button.install': 'Install', 'button.install_localization': 'Install Localization',
    'button.uninstall': 'Uninstall', 'button.uninstall_localization': 'Uninstall Localization', 'button.launch': 'Launch',
    'button.settings': 'Settings', 'button.update': 'Update', 'button.force_kill': 'Force Kill',
    'button.save': 'Save', 'button.cancel': 'Cancel', 'button.browse': 'Browse', 'button.confirm': 'Confirm',
    'button.select_language': 'Select Language',
    'status.localization_not_installed': 'Not installed', 'status.not_installed': 'Not installed', 'status.localization_installed': 'Installed',
    'status.no_compatible_version': 'No compatible version', 'status.up_to_date': 'Up to date', 'status.needs_update': 'Needs update',
    'status.running': 'Running', 'status.not_running': 'Not running', 'status.scanning': 'Scanning...',
    'status.no_instances': 'No Tanks Blitz instances detected', 'status.waiting_close': 'Waiting for game to close...',
    'progress.preparing': 'Preparing download', 'progress.l10n': 'Downloading localization pack', 'progress.font': 'Downloading font pack',
    'progress.extracting': 'Extracting', 'progress.extracted': 'Extraction complete', 'progress.installing_l10n': 'Installing localization pack',
    'progress.installing_font': 'Installing font pack', 'progress.done': 'Installation complete',
    'label.scan_added': 'Added {count} instance(s)', 'label.scan_no_new': 'No new instances found',
    'dialog.install_failed': 'Installation failed', 'dialog.install_failed_detail': 'Installation failed: {error}',
    'dialog.uninstall_failed': 'Uninstall failed', 'dialog.uninstall_success': 'Uninstalled successfully', 'dialog.uninstall_failed_detail': 'Uninstall failed: {error}',
    'dialog.settings_saved': 'Settings saved', 'dialog.kill_confirm': 'Game is running and must be closed first. Force kill?',
    'dialog.not_implemented': 'Not implemented', 'dialog.not_implemented_detail': 'Download and install will be available in a future version.',
    'label.game': 'Games', 'label.scanning': 'Scanning...', 'label.blitz_live': 'Tanks Blitz Live',
    'label.path': 'Path', 'label.version': 'Version', 'label.localization': 'Localization', 'label.font': 'Font', 'label.language': 'Language',
    'settings.language': 'Language', 'settings.theme': 'Theme', 'settings.proxy': 'Proxy Settings',
    'settings.use_system_proxy': 'Use system proxy', 'settings.custom_proxy': 'Custom proxy',
    'settings.host': 'Host', 'settings.port': 'Port', 'settings.username': 'Username', 'settings.password': 'Password',
    'settings.auth_optional': 'Authentication (Optional)', 'settings.proxy_host_required': 'Host is required', 'settings.proxy_port_required': 'Port is required',
    'settings.files': 'Files', 'settings.cache': 'Cache', 'settings.data_dir_label': 'Data Directory', 'settings.open_dir': 'Open Directory',
    'settings.clear_cache': 'Clear cache', 'settings.clear_cache_confirm_title': 'Confirm Clear Cache',
    'settings.clear_cache_confirm_msg': 'Download cache can significantly improve installation efficiency. But if your installed localization package cannot be read properly by the application, please try this option. Confirm clearing download cache?',
    'settings.clear_cache_success': 'Cache Cleared', 'label.latest': 'Latest',
    'about.version': 'Version', 'about.check_update': 'Check for updates', 'about.checking_update': 'Checking for updates…',
    'about.up_to_date': 'Up to date', 'about.update_available': 'New version v{version} available', 'about.update_now': 'Update Now',
    'about.update_download_progress': 'Downloading update… {percent}%', 'about.website': 'Official Website', 'about.github': 'GitHub Repository',
    'about.license': 'License', 'about.copyright': '© 2026 LocalizedBlitz',
    'error.update_check_failed': 'Update check failed', 'error.update_download_failed': 'Update download failed',
    'error.scan_failed': 'Scan failed', 'error.launch_failed': 'Launch failed', 'error.kill_failed': 'Force kill failed',
    'instance.path': 'Path', 'instance.version': 'Game version', 'instance.type': 'Type', 'instance.actions': 'Actions', 'instance.browse_manual': 'Browse manually',
  },
  ru: {
    'menu.home': 'Главная', 'menu.settings': 'Настройки', 'menu.about': 'О программе',
    'app.title_brand': 'Blitz Installer', 'app.title_sub': 'Установщик локализации для Tanks Blitz',
    'button.refresh': 'Обновить', 'button.auto_scan': 'Авто-сканирование', 'button.install': 'Установить', 'button.install_localization': 'Установить локализацию',
    'button.uninstall': 'Удалить', 'button.uninstall_localization': 'Удалить локализацию', 'button.launch': 'Запустить',
    'button.settings': 'Настройки', 'button.update': 'Обновить', 'button.force_kill': 'Завершить',
    'button.save': 'Сохранить', 'button.cancel': 'Отмена', 'button.browse': 'Обзор', 'button.confirm': 'Подтвердить',
    'button.select_language': 'Выбрать язык',
    'status.localization_not_installed': 'Не установлено', 'status.not_installed': 'Не установлено', 'status.localization_installed': 'Установлено',
    'status.no_compatible_version': 'Нет совместимой версии', 'status.up_to_date': 'Актуально', 'status.needs_update': 'Требуется обновление',
    'status.running': 'Запущена', 'status.not_running': 'Не запущена', 'status.scanning': 'Сканирование...',
    'status.no_instances': 'Экземпляры Tanks Blitz не найдены', 'status.waiting_close': 'Ожидание закрытия игры...',
    'progress.preparing': 'Подготовка загрузки', 'progress.l10n': 'Загрузка пакета локализации', 'progress.font': 'Загрузка шрифтового пакета',
    'progress.extracting': 'Распаковка', 'progress.extracted': 'Распаковка завершена', 'progress.installing_l10n': 'Установка пакета локализации',
    'progress.installing_font': 'Установка шрифтового пакета', 'progress.done': 'Установка завершена',
    'label.scan_added': 'Добавлено {count} экз.', 'label.scan_no_new': 'Новые экземпляры не найдены',
    'dialog.install_failed': 'Ошибка установки', 'dialog.install_failed_detail': 'Ошибка установки: {error}',
    'dialog.uninstall_failed': 'Ошибка удаления', 'dialog.uninstall_success': 'Удаление выполнено', 'dialog.uninstall_failed_detail': 'Ошибка удаления: {error}',
    'dialog.settings_saved': 'Настройки сохранены', 'dialog.kill_confirm': 'Игра запущена. Принудительно завершить?',
    'dialog.not_implemented': 'Не реализовано', 'dialog.not_implemented_detail': 'Загрузка и установка будут доступны в будущей версии.',
    'label.game': 'Игры', 'label.scanning': 'Сканирование...', 'label.blitz_live': 'Tanks Blitz Live',
    'label.path': 'Путь', 'label.version': 'Версия', 'label.localization': 'Локализация', 'label.font': 'Шрифт', 'label.language': 'Язык',
    'settings.language': 'Язык', 'settings.theme': 'Тема', 'settings.proxy': 'Настройки прокси',
    'settings.use_system_proxy': 'Системный прокси', 'settings.custom_proxy': 'Свой прокси',
    'settings.host': 'Хост', 'settings.port': 'Порт', 'settings.username': 'Пользователь', 'settings.password': 'Пароль',
    'settings.auth_optional': 'Аутентификация (необязательно)', 'settings.proxy_host_required': 'Хост обязателен', 'settings.proxy_port_required': 'Порт обязателен',
    'settings.files': 'Файлы', 'settings.cache': 'Кэш', 'settings.data_dir_label': 'Папка данных', 'settings.open_dir': 'Открыть папку',
    'settings.clear_cache': 'Очистить кэш', 'settings.clear_cache_confirm_title': 'Подтверждение очистки кэша',
    'settings.clear_cache_confirm_msg': 'Кэш загрузок значительно ускоряет установку. Но если установленный пакет локализации не читается приложением, попробуйте этот вариант. Подтвердить очистку кэша загрузок?',
    'settings.clear_cache_success': 'Кэш очищен', 'label.latest': 'Последняя',
    'about.version': 'Версия', 'about.check_update': 'Проверить обновления', 'about.checking_update': 'Проверка обновлений…',
    'about.up_to_date': 'Актуальная версия', 'about.update_available': 'Доступна новая версия v{version}', 'about.update_now': 'Обновить сейчас',
    'about.update_download_progress': 'Загрузка обновления… {percent}%', 'about.website': 'Официальный сайт', 'about.github': 'Репозиторий GitHub',
    'about.license': 'Лицензия', 'about.copyright': '© 2026 LocalizedBlitz',
    'error.update_check_failed': 'Ошибка проверки обновлений', 'error.update_download_failed': 'Ошибка загрузки обновления',
    'error.scan_failed': 'Ошибка сканирования', 'error.launch_failed': 'Ошибка запуска', 'error.kill_failed': 'Ошибка завершения',
    'instance.path': 'Путь', 'instance.version': 'Версия игры', 'instance.type': 'Тип', 'instance.actions': 'Действия', 'instance.browse_manual': 'Обзор вручную',
  },
};

export function getTranslation(key: string, lang: Language, params?: Record<string, string>): string {
  const map = translations[lang];
  let text = map?.[key] ?? translations['en']?.[key] ?? key;
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      text = text.replace(`{${k}}`, v);
    }
  }
  return text;
}
