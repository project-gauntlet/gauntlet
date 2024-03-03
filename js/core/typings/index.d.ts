declare module "gauntlet:renderer" {
    import { ReactNode } from "react";

    const render: (frontend: string, entrypointId: string, renderLocation: RenderLocation, component: ReactNode) => UiWidget;
    export { render };
}