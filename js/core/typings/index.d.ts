declare module "gauntlet:renderer" {
    import { FC } from "react";

    const render: (mode: "mutation" | "persistent", component: FC) => RootUiWidget;
    export { render };
}