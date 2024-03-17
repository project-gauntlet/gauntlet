// @ts-ignore TODO how to add declaration for this?
import { getAssetData, getAssetDataSync } from "gauntlet:renderer";

export async function assetData(path: string): Promise<ArrayBuffer> {
    return await getAssetData(path)
}

export function assetDataSync(path: string): ArrayBuffer {
    return getAssetDataSync(path)
}