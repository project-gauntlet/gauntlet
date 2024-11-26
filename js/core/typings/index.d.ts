declare module "ext:gauntlet/renderer.js" {
    import { ReactNode } from "react";

    export const render: (entrypointId: string, renderLocation: RenderLocation, component: ReactNode) => UiWidget;
    export const clearRenderer: () => void;
}

declare module "gauntlet:core" {
    export function runPluginLoop(): Promise<void>;
}

declare module "ext:gauntlet/api/components.js" {

}

declare module "ext:gauntlet/api/hooks.js" {

}

declare module "ext:gauntlet/api/helpers.js" {

}
