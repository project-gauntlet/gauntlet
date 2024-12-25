import {
    fetch_action_id_for_shortcut,
    get_command_generator_entrypoint_ids,
    op_log_info,
    op_log_debug,
    update_loading_bar
} from "ext:core/ops";
import { reloadSearchIndex } from "./search-index";
import type { FC } from "react";
import { renderView } from "./render";

interface GeneratedCommand { // TODO is it possible to import api here
    name: string
    actions: GeneratedCommandAction[]
    icon?: ArrayBuffer
    accessories?: GeneratedCommandAccessory[]
}

type GeneratedCommandAction = GeneratedCommandActionRun | GeneratedCommandActionView

interface GeneratedCommandActionRun {
    ref?: string
    label: string
    run: () => void
}

interface GeneratedCommandActionView {
    ref?: string
    label: string
    view: FC
}

type GeneratorProps = {
    add: (id: string, data: GeneratedCommand) => void,
    remove: (id: string) => void,
    get: (id: string) => GeneratedCommand | undefined
    getAll: () => GeneratedCommand[]
};

type Generator = (props: GeneratorProps) => void | (() => (void | Promise<void>)) | Promise<void | (() => (void | Promise<void>))>

type ProcessedGeneratedCommand = {
    generatorEntrypointId: string,
    uuid: string,
    command: GeneratedCommand
    derivedActions: GeneratedCommandDerivedAction[]
};

type GeneratedCommandDerivedAction = GeneratedCommandDerivedActionRun | GeneratedCommandDerivedActionView

interface GeneratedCommandDerivedActionRun {
    type: "Command"
    ref?: string
    label: string
    run: () => void
}

interface GeneratedCommandDerivedActionView {
    type: "View"
    ref?: string
    label: string
    view: FC
}


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

                if (data.actions.length < 1) {
                    throw new Error(`Error when adding entry '${id}': at least one action should be provided`)
                }

                const derivedActions: GeneratedCommandDerivedAction[] = []
                for (const action of data.actions) {
                    const label = action.label;

                    const run = "run" in action;
                    const view = "view" in action;

                    if (run && view) {
                        throw new Error(`only one of 'run' or 'view' properties can be specified in action: '${label}'`)
                    }

                    if (!run && !view) {
                        throw new Error(`one of 'run' or 'view' properties has to be specified in action: '${label}'`)
                    }

                    if (run) {
                        derivedActions.push({
                            type: "Command",
                            ref: action.ref,
                            label: action.label,
                            run: action.run,
                        })
                    } else if (view) {
                        derivedActions.push({
                            type: "View",
                            ref: action.ref,
                            label: action.label,
                            view: action.view,
                        })
                    }
                }

                const lookupId = generatorEntrypointId + ":" + id;

                storedGeneratedCommands[lookupId] = {
                    generatorEntrypointId: generatorEntrypointId,
                    uuid: crypto.randomUUID(),
                    command: data,
                    derivedActions,
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

export function generatedCommandSearchIndex(): GeneratedSearchItem[] {
    return Object.entries(storedGeneratedCommands).map(([entrypointLookupId, value]) => ({
        generator_entrypoint_id: value.generatorEntrypointId,
        entrypoint_id: entrypointLookupId,
        entrypoint_uuid: value.uuid,
        entrypoint_name: value.command.name,
        entrypoint_icon: value.command.icon,
        entrypoint_actions: value.derivedActions
            .map(action => ({
                id: action.ref,
                action_type: action.type,
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
            const action = command.derivedActions.find(value => value.ref == id);
            if (action) {
                runAction(entrypointId, action)
            }
        }
    }
}

export function runGeneratedCommand(entrypointId: string, action_index: number) {
    const generatedCommand = storedGeneratedCommands[entrypointId];

    if (generatedCommand) {
        const action = generatedCommand.derivedActions[action_index];
        if (action) {
            runAction(entrypointId, action)
        } else {
            throw new Error("Generated command with entrypoint id '" + entrypointId + "' doesn't have action with index: " + action_index)
        }
    } else {
        throw new Error("Generated command with entrypoint id '" + entrypointId + "' not found")
    }
}

function runAction(entrypointId: string, action: GeneratedCommandDerivedAction) {
    switch (action.type) {
        case "Command": {
            action.run()

            break;
        }
        case "View": {
            const entrypointName = storedGeneratedCommands[entrypointId]
                .command
                .name

            renderView(entrypointId, entrypointName, action.view)
            break;
        }
    }
}