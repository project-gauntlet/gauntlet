// @ts-ignore TODO how to add declaration for this?
import { getAssetData, getAssetDataSync, getPluginPreferences, getEntrypointPreferences, showHudWindow } from "ext:gauntlet/renderer.js";
import {
    clipboard_clear,
    clipboard_read,
    clipboard_read_text,
    clipboard_write,
    clipboard_write_text,
    environment_gauntlet_version,
    environment_is_development,
    environment_plugin_cache_dir,
    environment_plugin_data_dir
} from "ext:core/ops";

export function assetDataSync(path: string): ArrayBuffer {
    return getAssetDataSync(path)
}

export function assetData(path: string): Promise<ArrayBuffer> {
    return getAssetData(path)
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

export type GeneratedCommandAccessory = GeneratedCommandTextAccessory | GeneratedCommandIconAccessory;

export interface GeneratedCommandTextAccessory {
    text: string
    icon?: string
    tooltip?: string
}

export interface GeneratedCommandIconAccessory {
    icon: string
    tooltip?: string
}

export type GeneratorProps = {
    add: (id: string, data: GeneratedCommand) => void,
    remove: (id: string) => void,
    updateAccessories: (id: string, accessories: GeneratedCommandAccessory[] | undefined) => void,
};

export const Clipboard: Clipboard = {
    read: async function (): Promise<{ "text/plain"?: string | undefined; "image/png"?: ArrayBuffer | undefined; }> {
        const data = await clipboard_read();

        const result: { "text/plain"?: string; "image/png"?: ArrayBuffer; } = {};

        if (data.text_data) {
            result["text/plain"] = data.text_data;
        }

        if (data.png_data) {
            result["image/png"] = new Uint8Array(data.png_data).buffer;
        }

        return result
    },
    readText: async function (): Promise<string | undefined> {
        return await clipboard_read_text()
    },
    write: async function (data: { "text/plain"?: string | undefined; "image/png"?: ArrayBuffer | undefined; }): Promise<void> {
        const text_data = data["text/plain"];
        const png_data = data["image/png"];

        const write_data: { text_data?: string, png_data?: number[] } = {};

        if (text_data) {
            write_data.text_data = text_data;
        }

        if (png_data) {
            write_data.png_data = Array.from(new Uint8Array(png_data));
        }

        return await clipboard_write(write_data)
    },
    writeText: async function (data: string): Promise<void> {
        return await clipboard_write_text(data)
    },
    clear: async function (): Promise<void> {
        await clipboard_clear()
    }
}

export interface Clipboard {
    read(): Promise<{ ["text/plain"]?: string, ["image/png"]?: ArrayBuffer }>;
    readText(): Promise<string | undefined>;
    write(data: { ["text/plain"]?: string, ["image/png"]?: ArrayBuffer }): Promise<void>;
    writeText(data: string): Promise<void>;
    clear(): Promise<void>;
}

export const Environment: Environment = {
    get gauntletVersion(): number {
        return environment_gauntlet_version()
    },
    get isDevelopment(): boolean {
        return environment_is_development()
    },
    get pluginDataDir(): string {
        return environment_plugin_data_dir()
    },
    get pluginCacheDir(): string {
        return environment_plugin_cache_dir()
    },
}

export interface Environment {
    get gauntletVersion(): number;
    get isDevelopment(): boolean;
    get pluginDataDir(): string;
    get pluginCacheDir(): string;
}

