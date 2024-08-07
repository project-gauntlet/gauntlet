// @ts-expect-error does typescript support such symbol declarations?
const denoCore: DenoCore = Deno[Deno.internal].core;
const InternalApi = denoCore.ops;

interface GeneratedCommand { // TODO is it possible to import api here
    id: string
    name: string
    icon?: ArrayBuffer
    fn: () => void
}

type ProcessedGeneratedCommand = GeneratedCommand & { lookupId: string, uuid: string };

let storedGeneratedCommands: ProcessedGeneratedCommand[] = []

export async function runCommandGenerators(): Promise<void> {
    let localGeneratedCommands: ProcessedGeneratedCommand[] = []

    const entrypointIds = await InternalApi.get_command_generator_entrypoint_ids();
    for (const generatorEntrypointId of entrypointIds) {
        try {
            const generator: () => Promise<GeneratedCommand[]> = (await import(`gauntlet:entrypoint?${generatorEntrypointId}`)).default;

            InternalApi.op_log_info("command_generator", `Running command generator for entrypoint ${generatorEntrypointId}`)

            const generatedCommands = (await generator())
                .map(value => {
                    return {
                        lookupId: generatorEntrypointId + ":" + value.id,
                        uuid: crypto.randomUUID(),
                        ...value
                    }
                });

            InternalApi.op_log_info("command_generator", `Finished running command generator for entrypoint ${generatorEntrypointId}, amount: ${generatedCommands.length}`)

            localGeneratedCommands.push(...generatedCommands)
        } catch (e) {
            console.error("Error occurred when calling command generator for entrypoint: " + generatorEntrypointId, e)
        }
    }

    storedGeneratedCommands = localGeneratedCommands
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