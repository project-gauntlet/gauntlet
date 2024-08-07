import {GeneratedCommand} from "@project-gauntlet/api/helpers";

interface DesktopEntry {
    name: string,
    icon: ArrayBuffer | undefined,
    command: string[],
}

// @ts-expect-error
const denoCore: DenoCore = Deno[Deno.internal].core;
const InternalApi: InternalApi = denoCore.ops;

interface InternalApi {
    list_applications(): Promise<DesktopEntry[]>
    open_application(command: string[]): void
}

export default async function Default(): Promise<GeneratedCommand[]> {
    return (await InternalApi.list_applications())
        .map(value => ({
            id: `${value.name}-${value.command.join("-")}`,
            name: value.name,
            icon: value.icon,
            fn: () => {
                InternalApi.open_application(value.command)
            }
        }));
}
