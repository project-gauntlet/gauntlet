// @ts-ignore TODO how to add declaration for this?
import { getAssetData, getAssetDataSync, getPluginPreferences, getEntrypointPreferences } from "gauntlet:renderer";

export async function assetData(path: string): Promise<ArrayBuffer> {
    return await getAssetData(path)
}

export function assetDataSync(path: string): ArrayBuffer {
    return getAssetDataSync(path)
}

export function pluginPreferences<T extends Record<string, any>>(): T {
    return getPluginPreferences()
}

export function entrypointPreferences<T extends Record<string, any>>(): T {
    return getEntrypointPreferences()
}