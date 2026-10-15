import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export async function checkAndDownloadAssets(
  onProgress: (text: string) => void
) {
  try {
    onProgress('Checking for asset updates...');
    const info = await invoke<any>('check_assets_update');

    if (info.has_update) {
      onProgress('Downloading new assets... 0%');

      const unlistenProgress = await listen<number>('asset-download-progress', (e) => {
        onProgress(`Downloading new assets... ${Math.round(e.payload)}%`);
      });

      const unlistenExtract = await listen('asset-extract-start', () => {
        onProgress('Extracting assets...');
      });

      await invoke('download_assets', { remoteVersion: info.remote_version });

      unlistenProgress();
      unlistenExtract();
      onProgress('Assets updated!');
    }
  } catch (error) {
    console.error('Failed to update assets:', error);
    let msg = typeof error === 'string' ? error : 'Unknown error';
    onProgress('Asset error: ' + msg);
    await new Promise((resolve) => setTimeout(resolve, 5000));
  }
}
