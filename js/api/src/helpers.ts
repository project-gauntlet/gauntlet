// @ts-ignore TODO how to add declaration for this?
import { getAssetDataSync, getPluginPreferences, getEntrypointPreferences, showHudWindow } from "gauntlet:renderer";

// @ts-expect-error does typescript support such symbol declarations?
const denoCore: DenoCore = Deno[Deno.internal].core;
const InternalApi = denoCore.ops;

export function assetData(path: string): ArrayBuffer {
    return getAssetDataSync(path)
}

export function pluginPreferences<T extends Record<string, any>>(): T {
    return getPluginPreferences()
}

export function entrypointPreferences<T extends Record<string, any>>(): T {
    return getEntrypointPreferences()
}

export function showHud(display: string): void {
    return showHudWindow(display)
}

export interface GeneratedCommand {
    name: string
    icon?: ArrayBuffer
    fn: () => void
    actions?: GeneratedCommandAction[]
}

export interface GeneratedCommandAction {
    ref?: string
    label: string
    fn: () => void
}

export type GeneratorProps = {
    add: (id: string, data: GeneratedCommand) => void,
    remove: (id: string) => void,
};

export const Clipboard: Clipboard = {
    read: async function (): Promise<{ "text/plain"?: string | undefined; "image/png"?: Blob | undefined; }> {
        const data = await InternalApi.clipboard_read();

        const result: { "text/plain"?: string; "image/png"?: Blob; } = {};

        if (data.text_data) {
            result["text/plain"] = data.text_data;
        }

        if (data.png_data) {
            result["image/png"] = data.png_data;  // TODO arraybuffer? fix when migrating to deno's op2
        }

        return result
    },
    readText: async function (): Promise<string | undefined> {
        return await InternalApi.clipboard_read_text()
    },
    write: async function (data: { "text/plain"?: string | undefined; "image/png"?: Blob | undefined; }): Promise<void> {
        const text_data = data["text/plain"];
        const png_data = data["image/png"];
        return await InternalApi.clipboard_write({
            text_data: text_data,
            png_data: png_data != undefined ? Array.from(new Uint8Array(png_data as any)) : undefined, // TODO arraybuffer? fix when migrating to deno's op2
        })
    },
    writeText: async function (data: string): Promise<void> {
        return await InternalApi.clipboard_write_text(data)
    },
    clear: async function (): Promise<void> {
        await InternalApi.clipboard_clear()
    }
}

export interface Clipboard {
    read(): Promise<{ ["text/plain"]?: string, ["image/png"]?: Blob }>;
    readText(): Promise<string | undefined>;
    write(data: { ["text/plain"]?: string, ["image/png"]?: Blob }): Promise<void>;
    writeText(data: string): Promise<void>;
    clear(): Promise<void>;
}
