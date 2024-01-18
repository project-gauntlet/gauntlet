declare module "gauntlet:renderer" {
    import { FC } from "react";

    const render: (frontend: string, component: FC) => UiWidget;
    export { render };
}