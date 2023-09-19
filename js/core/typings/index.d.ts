
// TODO autogenerate typings from another projects
declare module "plugin:view" {
    import { FC } from "react";

    const view: FC;
    export default view;
}

declare module "plugin:renderer" {
    import { FC } from "react";

    const render: (mode: "mutation" | "persistent", component: FC) => void;
    export { render };
}