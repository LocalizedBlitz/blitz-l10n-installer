<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { listen } from "@tauri-apps/api/event";
  import { getTranslation, LANGUAGE_DISPLAY, type Language } from "$lib/i18n";

  let {
    lang = 'zh_CN',
    runningStates = {} as Record<string, boolean>,
    blitzPaths = [] as string[],
    onPathsChange,
    onRefreshRunning,
  }: {
    lang?: Language;
    runningStates?: Record<string, boolean>;
    blitzPaths?: string[];
    onPathsChange?: (paths: string[]) => void;
    onRefreshRunning?: () => Promise<void>;
  } = $props();

  interface InstanceInfo {
    path: string;
    type: string;
  }

  interface BlitzStatus {
    path: string;
    version: string;
    loc_installed: boolean;
    loc_version: string;
    font_version: string;
    loc_language: string;
    is_compatible: boolean;
  }

  interface ProgressPayload {
    step: string;
    percent: number;
    message_key: string;
    instance: string;
    downloaded_bytes: number;
    total_bytes: number;
  }

  interface ProgressState {
    visible: boolean;
    percent: number;
    message: string;
    downloadedBytes: number;
    totalBytes: number;
    isDownload: boolean;
  }

  let instances: InstanceInfo[] = $state([]);
  let statuses: Record<string, BlitzStatus> = $state({});
  let scanning: boolean = $state(false);
  let refreshing: boolean = $state(false);
  let toastMessage: string | null = $state(null);
  let toastType: 'success' | 'error' | 'warning' | 'info' = $state('info');

  let showKillConfirm: boolean = $state(false);
  let killTargetPath: string = $state('');
  let pendingKillAction: (() => Promise<void>) | null = $state(null);

  let installProgress: ProgressState = $state({ visible: false, percent: 0, message: '', downloadedBytes: 0, totalBytes: 0, isDownload: false });
  let blitzLangId: string = $state('zh_CN');

  let installingPath: string = $state('');

  function t(key: string, params?: Record<string, string>): string { return getTranslation(key, lang, params); }

  function formatBytes(bytes: number): string {
    if (bytes >= 1048576) { const v = (bytes / 1048576).toFixed(1); return (v.endsWith('.0') ? v.slice(0, -2) : v) + ' MB'; }
    if (bytes >= 1024) { const v = (bytes / 1024).toFixed(1); return (v.endsWith('.0') ? v.slice(0, -2) : v) + ' KB'; }
    return bytes + ' B';
    return bytes + ' B';
  }

  function showToast(message: string, type: 'success' | 'error' | 'warning' | 'info' = 'info') {
    toastMessage = message;
    toastType = type;
    setTimeout(() => { toastMessage = null; }, 4000);
  }

  // Install progress listener
  let unlistenProgress: (() => void) | null = null;
  $effect(() => {
    listen<ProgressPayload>('install-progress', (event) => {
      const p = event.payload;
      installProgress = {
        visible: p.step !== 'done',
        percent: p.percent,
        message: p.message_key ? t('progress.' + p.message_key) : '',
        downloadedBytes: p.downloaded_bytes > 0 ? p.downloaded_bytes : installProgress.downloadedBytes,
        totalBytes: p.total_bytes > 0 ? p.total_bytes : installProgress.totalBytes,
        isDownload: p.step === 'download',
      };
      if (p.step === 'done') {
        refreshStatuses();
        showToast(t('status.localization_installed'), 'success');
        installingPath = '';
      }
    }).then(fn => { unlistenProgress = fn; });
    return () => { unlistenProgress?.(); };
  });

  async function scan() {
    scanning = true;
    const prevCount = instances.length;
    try {
      const result = await invoke<InstanceInfo[]>('scan_instances');
      instances = result;
      const paths = result.map(i => i.path);
      onPathsChange?.(paths);
      try {
        const config = await invoke<{ blitz_paths: string[] }>('get_app_config');
        config.blitz_paths = paths;
        await invoke('save_app_config', { appConfig: config });
      } catch {}
      await refreshStatuses();
      const newCount = result.length - prevCount;
      if (result.length === 0) {
        showToast(t('status.no_instances'), 'warning');
      } else if (prevCount > 0) {
        // Only show delta on re-scans, not on initial load
        if (newCount > 0) {
          showToast(t('label.scan_added', { count: String(newCount) }), 'success');
        } else {
          showToast(t('label.scan_no_new'), 'info');
        }
      }
    } catch (e) {
      showToast(t('error.scan_failed'), 'error');
    }
    scanning = false;
  }

  async function refreshAll() {
    refreshing = true;
    await scan();
    refreshing = false;
  }

  async function refreshStatuses() {
    const map: Record<string, BlitzStatus> = {};
    for (const inst of instances) {
      try {
        map[inst.path] = await invoke<BlitzStatus>('check_blitz_status', { blitzPath: inst.path });
      } catch {
        map[inst.path] = { path: inst.path, version: '', loc_installed: false, loc_version: '', font_version: '', loc_language: '', is_compatible: false };
      }
    }
    statuses = map;
  }

  async function loadLangId() {
    try {
      const config = await invoke<{ blitz_lang_id: string }>('get_app_config');
      if (config.blitz_lang_id) blitzLangId = config.blitz_lang_id;
    } catch {}
  }

  async function addManualPath() {
    const selected = await open({ directory: true, title: 'Select Tanks Blitz directory' });
    if (!selected) return;
    const path = selected as string;
    try {
      const valid = await invoke<boolean>('validate_instance_path', { path });
      if (!valid) { showToast('Invalid Tanks Blitz directory', 'error'); return; }
      if (instances.some(i => i.path === path)) return;
      instances = [...instances, { path, type: 'production' }];
      const paths = instances.map(i => i.path);
      onPathsChange?.(paths);
      try {
        const config = await invoke<{ blitz_paths: string[] }>('get_app_config');
        config.blitz_paths = paths;
        await invoke('save_app_config', { appConfig: config });
      } catch {}
      await refreshStatuses();
    } catch { showToast('Validation failed', 'error'); }
  }

  async function launchGame(path: string) {
    try {
      const exePath = path.replace(/\\/g, '/') + '/tanksblitz.exe';
      await invoke('launch_app', { path: exePath });
      await onRefreshRunning?.();
    } catch { showToast(t('error.launch_failed'), 'error'); }
  }

  async function forceKillGame(path: string) {
    try {
      await invoke('force_kill_app', { installPath: path });
      await onRefreshRunning?.();
    } catch { showToast(t('error.kill_failed'), 'error'); }
  }

  function withKillCheck(path: string, action: () => Promise<void>) {
    if (runningStates[path]) {
      killTargetPath = path;
      pendingKillAction = action;
      showKillConfirm = true;
    } else { action(); }
  }

  async function onKillConfirm() {
    showKillConfirm = false;
    if (killTargetPath) {
      await forceKillGame(killTargetPath);
      if (pendingKillAction) {
        setTimeout(async () => { await pendingKillAction!(); pendingKillAction = null; }, 1000);
      }
    }
  }

  function onKillCancel() { showKillConfirm = false; pendingKillAction = null; }

  async function onInstall(path: string) {
    installingPath = path;
    try {
      await invoke<string>('download_and_install_blitz', { blitzPath: path, langId: blitzLangId });
    } catch (e: any) {
      installingPath = '';
      showToast(t('dialog.install_failed_detail', { error: String(e) }), 'error');
    }
  }

  async function onUninstall(path: string) {
    try {
      await invoke<string>('full_uninstall', { path });
      showToast(t('dialog.uninstall_success'), 'success');
      await refreshStatuses();
    } catch (e: any) {
      showToast(t('dialog.uninstall_failed_detail', { error: String(e) }), 'error');
    }
  }

  let initialized = false;
  $effect(() => { if (!initialized) { initialized = true; loadLangId(); scan(); } });
</script>

<div class="flex flex-col gap-4 h-full">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <h2 class="text-xl font-bold">{t('label.game')}</h2>
    <div class="flex gap-2">
      <button class="btn btn-sm btn-outline" onclick={refreshAll} disabled={refreshing}>
        {#if refreshing}<svg class="animate-spin h-3 w-3 text-muted" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"/></svg>{/if}
        {t('button.refresh')}
      </button>
      <button class="btn btn-sm btn-primary" onclick={scan} disabled={scanning}>
        {#if scanning}<svg class="animate-spin h-3 w-3 text-muted" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"/></svg>{/if}
        {scanning ? t('label.scanning') : t('button.auto_scan')}
      </button>
      <button class="btn btn-sm btn-outline" onclick={addManualPath}>{t('instance.browse_manual')}</button>
    </div>
  </div>

  <!-- Instance cards -->
  {#if instances.length === 0 && !scanning}
    <div class="card card-body text-center py-10">
      <p class="text-muted">{t('status.no_instances')}</p>
    </div>
  {:else}
    {#each instances as inst (inst.path)}
      {@const status = statuses[inst.path]}
      {@const running = runningStates[inst.path] ?? false}
      <div class="card">
        <div class="card-body p-5">
          <div class="flex items-center justify-between mb-3">
            <div class="flex items-center gap-2">
              <h3 class="card-title text-lg m-0">{t('label.blitz_live')}</h3>
              {#if running}
                <span class="badge badge-warning">{t('status.running')}</span>
              {/if}
              {#if status?.loc_installed}
                {#if status.is_compatible}
                  <span class="badge badge-success">{t('status.localization_installed')}</span>
                {:else}
                  <span class="badge badge-muted">{t('status.no_compatible_version')}</span>
                {/if}
              {:else}
                <span class="badge badge-muted">{t('status.localization_not_installed')}</span>
              {/if}
            </div>
            {#if instances.length > 1}
              <span class="text-xs text-muted truncate max-w-[50%]" title={inst.path}>{inst.path}</span>
            {/if}
          </div>

          <div class="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-sm">
            <span class="text-muted">{t('label.path')}:</span>
            <span class="truncate">{inst.path || '-'}</span>
            <span class="text-muted">{t('label.version')}:</span>
            <span>{status?.version || '-'}</span>
            <span class="text-muted">{t('label.localization')}:</span>
            <span>
              {#if status?.loc_installed}
                {status.loc_version || '?'}
              {:else}
                {t('status.not_installed')}
              {/if}
            </span>
            <span class="text-muted">{t('label.font')}:</span>
            <span>
              {#if status?.loc_installed && status.font_version}
                {status.font_version}
              {:else if status?.loc_installed}
                -
              {:else}
                {t('status.not_installed')}
              {/if}
            </span>
            <span class="text-muted">{t('label.language')}:</span>
            <span>
              {#if status?.loc_installed}
                {LANGUAGE_DISPLAY[status.loc_language as Language] || status.loc_language}
              {:else}
                {t('status.not_installed')}
              {/if}
            </span>
          </div>

          {#if installProgress.visible && installingPath === inst.path}
            <div class="mt-2">
              <div class="flex items-center justify-between mb-1">
                <span class="text-xs font-medium">{installProgress.message}</span>
                <span class="text-xs text-muted">{installProgress.percent}%</span>
              </div>
              <progress class="progress w-full" value={installProgress.percent} max="100"></progress>
              {#if installProgress.isDownload && installProgress.totalBytes > 0}
                <p class="text-xs text-muted mt-1">
                  {formatBytes(installProgress.downloadedBytes)} / {formatBytes(installProgress.totalBytes)}
                </p>
              {/if}
            </div>
          {/if}

          <div class="card-actions justify-end mt-3">
            {#if running}
              <button class="btn btn-sm btn-danger" onclick={() => forceKillGame(inst.path)}>
                {t('button.force_kill')}
              </button>
            {:else}
              <button class="btn btn-sm btn-outline" onclick={() => launchGame(inst.path)}>
                {t('button.launch')}
              </button>
              {#if status?.loc_installed}
                <button class="btn btn-sm btn-danger" onclick={() => withKillCheck(inst.path, () => onUninstall(inst.path))}>
                  {t('button.uninstall_localization')}
                </button>
              {:else}
                <button class="btn btn-sm btn-primary" onclick={() => withKillCheck(inst.path, () => onInstall(inst.path))} disabled={installingPath !== ''}>
                  {#if installingPath === inst.path}
                    <svg class="animate-spin h-3 w-3 text-muted" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"/></svg>
                  {/if}
                  {t('button.install_localization')}
                </button>
              {/if}
            {/if}
          </div>
        </div>
      </div>
    {/each}
  {/if}
</div>

{#if showKillConfirm}
  <div class="modal-backdrop" onclick={onKillCancel} role="dialog" onkeydown={(e: KeyboardEvent) => e.key === 'Escape' && onKillCancel()}>
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="modal-box" onclick={(e: MouseEvent) => e.stopPropagation()}>
      <p class="text-sm mb-4">{t('dialog.kill_confirm')}</p>
      <div class="modal-action">
        <button class="btn btn-outline btn-sm" onclick={onKillCancel}>{t('button.cancel')}</button>
        <button class="btn btn-danger btn-sm" onclick={onKillConfirm}>{t('button.force_kill')}</button>
      </div>
    </div>
  </div>
{/if}

{#if toastMessage}
  <div class="toast-container">
    <div class="toast toast-{toastType}">
      <span>{toastMessage}</span>
    </div>
  </div>
{/if}
