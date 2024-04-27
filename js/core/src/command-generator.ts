// @ts-expect-error does typescript support such symbol declarations?
const denoCore: DenoCore = Deno[Deno.internal].core;
const InternalApi = denoCore.ops;

interface GeneratedCommand { // TODO Add this type to api
    id: string
    name: string
    icon: ArrayBuffer | undefined
    fn: () => void
}

let storedGeneratedCommands: (GeneratedCommand & { lookupId: string, uuid: string })[] = []

export async function runCommandGenerators(): Promise<void> {
    storedGeneratedCommands = []

    const entrypointIds = await InternalApi.get_command_generator_entrypoint_ids();
    for (const generatorEntrypointId of entrypointIds) {
        try {
            const generator: () => GeneratedCommand[] = (await import(`gauntlet:entrypoint?${generatorEntrypointId}`)).default;

            const generatedCommands = generator()
                .map(value => {
                    return {
                        lookupId: generatorEntrypointId + ":" + value.id,
                        uuid: crypto.randomUUID(),
                        ...value
                    }
                });

            storedGeneratedCommands.push(...generatedCommands)
        } catch (e) {
            console.error("Error occurred when calling command generator for entrypoint: " + generatorEntrypointId, e)
        }
    }
}

export function generatedCommandSearchIndex(): AdditionalSearchItem[] {
    return storedGeneratedCommands.map(value => ({
        entrypoint_id: value.lookupId,
        entrypoint_uuid: value.uuid,
        entrypoint_name: value.name,
        entrypoint_icon: value.icon,
    }))
}

export function runGeneratedCommand(entrypointId: string) {
    const generatedCommand = storedGeneratedCommands.find(value => value.lookupId === entrypointId);

    if (generatedCommand) {
        generatedCommand.fn()
    } else {
        throw new Error("Generated command with entrypoint id '" + entrypointId + "' not found")
    }
}