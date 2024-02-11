declare module "gauntlet:renderer" {
    import { FC } from "react";

    const renderTopmostView: (frontend: string, component: FC) => UiWidget;
    export { renderTopmostView };
}