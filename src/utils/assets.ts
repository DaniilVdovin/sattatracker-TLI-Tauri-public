import { invoke } from '@tauri-apps/api/core';
import { convertFileSrc } from '@tauri-apps/api/core';
import { ref } from 'vue';

export const assetsState = ref({
    basePath: '',
});

export async function initAssets() {
    try {
        const path: string = await invoke('get_assets_path');
        assetsState.value.basePath = path;
    } catch (e) {
        console.error('Failed to get assets path', e);
    }
}

export function getIcon(id: string | number) {
    if (!assetsState.value.basePath) return '';
    return convertFileSrc(`${assetsState.value.basePath}\\icons\\${id}.png`);
}

export function getAnim(subpath: string) {
    if (!assetsState.value.basePath) return '';
    return convertFileSrc(`${assetsState.value.basePath}\\piganim\\${subpath}.png`);
}
