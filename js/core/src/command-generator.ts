import {
    fetch_action_id_for_shortcut,
    get_command_generator_entrypoint_ids,
    op_log_info,
    op_log_debug,
    update_loading_bar
} from "ext:core/ops";
import { reloadSearchIndex } from "./search-index";

interface GeneratedCommand { // TODO is it possible to import api here
    name: string
    icon?: ArrayBuffer
    fn: () => void
    actions?: GeneratedCommandAction[]
    accessories?: GeneratedCommandAccessory[]
}

interface GeneratedCommandAction {
    ref?: string
    label: string
    fn: () => void
}

type GeneratorProps = {
    add: (id: string, data: GeneratedCommand) => void,
    remove: (id: string) => void,
    get: (id: string) => GeneratedCommand | undefined
    getAll: () => GeneratedCommand[]
};

type Generator = (props: GeneratorProps) => void | (() => (void | Promise<void>)) | Promise<void | (() => (void | Promise<void>))>

type ProcessedGeneratedCommand = { generatorEntrypointId: string, uuid: string, command: GeneratedCommand };

type ProcessedGeneratedCommands = { [lookupEntrypointId: string]: ProcessedGeneratedCommand };
type GeneratorCleanups = { [generatorEntrypointId: string]: () => (void | Promise<void>) };

let storedGeneratedCommands: ProcessedGeneratedCommands = {}
let generatorCleanups: GeneratorCleanups = {}

export async function runCommandGenerators(): Promise<void> {
    for (let [generatorEntrypointId, cleanup] of Object.entries(generatorCleanups)) {
        try {
            await cleanup()
        } catch (err) {
            console.error(`Error occurred when calling cleanup function of generator entrypoint: ${generatorEntrypointId}`, err)
        }
    }

    storedGeneratedCommands = {}
    generatorCleanups = {}

    await reloadSearchIndex(true)

    const entrypointIds = await get_command_generator_entrypoint_ids();
    for (const generatorEntrypointId of entrypointIds) {
        try {
            const generator: Generator = (await import(`gauntlet:entrypoint?${generatorEntrypointId}`)).default;

            op_log_info("command_generator", `Running command generator entrypoint ${generatorEntrypointId}`)

            const add = (id: string, data: GeneratedCommand) => {
                op_log_info("command_generator", `Adding entry '${id}' by command generator entrypoint '${generatorEntrypointId}'`)

                const lookupId = generatorEntrypointId + ":" + id;

                storedGeneratedCommands[lookupId] = {
                    generatorEntrypointId: generatorEntrypointId,
                    uuid: crypto.randomUUID(),
                    command: data,
                }

                reloadSearchIndex(true)
            }
            const remove = (id: string) => {
                op_log_info("command_generator", `Removing entry '${id}' by command generator entrypoint '${generatorEntrypointId}'`)
                const lookupId = generatorEntrypointId + ":" + id;

                delete storedGeneratedCommands[lookupId]

                reloadSearchIndex(true)
            }

            const get = (id: string) => {
                op_log_debug("command_generator", `Getting entry '${id}' by command generator entrypoint '${generatorEntrypointId}'`)
                const lookupId = generatorEntrypointId + ":" + id;

                const generatedCommand = storedGeneratedCommands[lookupId];
                if (generatedCommand) {
                    return generatedCommand.command
                } else {
                    return undefined
                }
            }

            const getAll = (): GeneratedCommand[] => {
                op_log_debug("command_generator", `Getting all entries by command generator entrypoint '${generatorEntrypointId}'`)

                return Object.entries(storedGeneratedCommands)
                    .map(([_, value]) => value.command)
            }

            // noinspection ES6MissingAwait
            (async () => {
                try {
                    update_loading_bar(generatorEntrypointId, true)
                    let cleanup = await generator({ add, remove, get, getAll })
                    update_loading_bar(generatorEntrypointId, false)
                    if (typeof cleanup === "function") {
                        generatorCleanups[generatorEntrypointId] = cleanup
                    }
                } catch (e) {
                    console.error(`Error occurred when calling command generator for entrypoint: ${generatorEntrypointId}`, e)
                }
            })()
        } catch (e) {
            console.error(`Error occurred when importing command generator for entrypoint: ${generatorEntrypointId}`, e)
        }
    }
}

export function generatedCommandSearchIndex(): AdditionalSearchItem[] {
    return Object.entries(storedGeneratedCommands).map(([entrypointLookupId, value]) => ({
        generator_entrypoint_id: value.generatorEntrypointId,
        entrypoint_id: entrypointLookupId,
        entrypoint_uuid: value.uuid,
        entrypoint_name: value.command.name,
        entrypoint_icon: value.command.icon,
        entrypoint_actions: (value.command.actions || [])
            .map(action => ({
                id: action.ref,
                label: action.label
            })),
        entrypoint_accessories: value.command.accessories || []
    }))
}

export async function runGeneratedCommandAction(entrypointId: string, key: string, modifierShift: boolean, modifierControl: boolean, modifierAlt: boolean, modifierMeta: boolean) {
    const command = storedGeneratedCommands[entrypointId];

    if (command) {
        const id = await fetch_action_id_for_shortcut(command.generatorEntrypointId, key, modifierShift, modifierControl, modifierAlt, modifierMeta);
        if (id) {
            const action = command.command.actions?.find(value => value.ref == id);
            if (action) {
                action.fn()
            }
        }
    }
}

export function runGeneratedCommand(entrypointId: string, action_index: number | undefined) {
    const generatedCommand = storedGeneratedCommands[entrypointId];

    if (generatedCommand) {
        if (typeof action_index == "number") {
            const actions = generatedCommand.command.actions;
            if (actions) {
                actions[action_index].fn()
            } else {
                throw new Error("Generated command with entrypoint id '" + entrypointId + "' doesn't have actions, action index: " + action_index)
            }
        } else {
            generatedCommand.command.fn()
        }
    } else {
        throw new Error("Generated command with entrypoint id '" + entrypointId + "' not found")
    }
}