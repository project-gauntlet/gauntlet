// @ts-ignore TODO how to add declaration for this?
import { getAssetDataSync, getPluginPreferences, getEntrypointPreferences } from "gauntlet:renderer";

export function assetData(path: string): ArrayBuffer {
    return getAssetDataSync(path)
}

export function pluginPreferences<T extends Record<string, any>>(): T {
    return getPluginPreferences()
}

export function entrypointPreferences<T extends Record<string, any>>(): T {
    return getEntrypointPreferences()
}