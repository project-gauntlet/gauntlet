import {
    fetch_action_id_for_shortcut,
    get_entrypoint_generator_entrypoint_ids,
    op_log_info,
    op_log_debug,
    update_loading_bar
} from "ext:core/ops";
import { reloadSearchIndex } from "./search-index";
import type { FC } from "react";
import { renderView } from "./render";

interface GeneratedEntrypoint { // TODO is it possible to import api here
    name: string
    actions: GeneratedEntrypointAction[]
    icon?: ArrayBuffer
    accessories?: GeneratedEntrypointAccessory[]
}

type GeneratedEntrypointAction = GeneratedEntrypointActionRun | GeneratedEntrypointActionView

interface GeneratedEntrypointActionRun {
    ref?: string
    label: string
    run: () => void
}

interface GeneratedEntrypointActionView {
    ref?: string
    label: string
    view: FC
}

type GeneratorProps = {
    add: (id: string, data: GeneratedEntrypoint) => void,
    remove: (id: string) => void,
    get: (id: string) => GeneratedEntrypoint | undefined
    getAll: () => { [id: string]: GeneratedEntrypoint }
};

type Generator = (props: GeneratorProps) => void | (() => (void | Promise<void>)) | Promise<void | (() => (void | Promise<void>))>

type ProcessedGeneratedEntrypoint = {
    generatorEntrypointId: string,
    id: string,
    uuid: string,
    command: GeneratedEntrypoint
    derivedActions: GeneratedEntrypointDerivedAction[]
};

type GeneratedEntrypointDerivedAction = GeneratedEntrypointDerivedActionRun | GeneratedEntrypointDerivedActionView

interface GeneratedEntrypointDerivedActionRun {
    type: "Command"
    ref?: string
    label: string
    run: () => void
}

interface GeneratedEntrypointDerivedActionView {
    type: "View"
    ref?: string
    label: string
    view: FC
}


type ProcessedGeneratedEntrypoints = { [lookupEntrypointId: string]: ProcessedGeneratedEntrypoint };
type GeneratorCleanups = { [generatorEntrypointId: string]: () => (void | Promise<void>) };

let storedGeneratedEntrypoints: ProcessedGeneratedEntrypoints = {}
let generatorCleanups: GeneratorCleanups = {}

export async function runEntrypointGenerators(): Promise<void> {
    for (let [generatorEntrypointId, cleanup] of Object.entries(generatorCleanups)) {
        try {
            await cleanup()
        } catch (err) {
            console.error(`Error occurred when calling cleanup function of generator entrypoint: ${generatorEntrypointId}`, err)
        }
    }

    storedGeneratedEntrypoints = {}
    generatorCleanups = {}

    await reloadSearchIndex(true)

    const entrypointIds = await get_entrypoint_generator_entrypoint_ids();
    for (const generatorEntrypointId of entrypointIds) {
        try {
            const generator: Generator = (await import(`gauntlet:entrypoint?${generatorEntrypointId}`)).default;

            op_log_info("entrypoint_generator", `Running entrypoint generator entrypoint ${generatorEntrypointId}`)

            const add = (id: string, data: GeneratedEntrypoint) => {
                op_log_info("entrypoint_generator", `Adding entry '${id}' by entrypoint generator entrypoint '${generatorEntrypointId}'`)

                if (data.actions.length < 1) {
                    throw new Error(`Error when adding entry '${id}': at least one action should be provided`)
                }

                const derivedActions: GeneratedEntrypointDerivedAction[] = []
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

                storedGeneratedEntrypoints[lookupId] = {
                    generatorEntrypointId: generatorEntrypointId,
                    id: id,
                    uuid: crypto.randomUUID(),
                    command: data,
                    derivedActions,
                }

                reloadSearchIndex(true)
            }
            const remove = (id: string) => {
                op_log_info("entrypoint_generator", `Removing entry '${id}' by entrypoint generator entrypoint '${generatorEntrypointId}'`)
                const lookupId = generatorEntrypointId + ":" + id;

                delete storedGeneratedEntrypoints[lookupId]

                reloadSearchIndex(true)
            }

            const get = (id: string) => {
                op_log_debug("entrypoint_generator", `Getting entry '${id}' by entrypoint generator entrypoint '${generatorEntrypointId}'`)
                const lookupId = generatorEntrypointId + ":" + id;

                const generatedEntrypoint = storedGeneratedEntrypoints[lookupId];
                if (generatedEntrypoint) {
                    return generatedEntrypoint.command
                } else {
                    return undefined
                }
            }

            const getAll = (): { [id: string]: GeneratedEntrypoint } => {
                op_log_debug("entrypoint_generator", `Getting all entries by entrypoint generator entrypoint '${generatorEntrypointId}'`)

                return Object.fromEntries(
                    Object.entries(storedGeneratedEntrypoints)
                        .map(([_lookupId, value]) => [value.id, value.command])
                )
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
                    console.error(`Error occurred when calling entrypoint generator for entrypoint: ${generatorEntrypointId}`, e)
                }
            })()
        } catch (e) {
            console.error(`Error occurred when importing entrypoint generator for entrypoint: ${generatorEntrypointId}`, e)
        }
    }
}

export function generatedEntrypointSearchIndex(): GeneratedSearchItem[] {
    return Object.entries(storedGeneratedEntrypoints).map(([entrypointLookupId, value]) => ({
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

export async function runGeneratedEntrypointAction(entrypointId: string, key: string, modifierShift: boolean, modifierControl: boolean, modifierAlt: boolean, modifierMeta: boolean) {
    const command = storedGeneratedEntrypoints[entrypointId];

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

export function runGeneratedEntrypoint(entrypointId: string, action_index: number) {
    const generatedEntrypoint = storedGeneratedEntrypoints[entrypointId];

    if (generatedEntrypoint) {
        const action = generatedEntrypoint.derivedActions[action_index];
        if (action) {
            runAction(entrypointId, action)
        } else {
            throw new Error("Generated command with entrypoint id '" + entrypointId + "' doesn't have action with index: " + action_index)
        }
    } else {
        throw new Error("Generated command with entrypoint id '" + entrypointId + "' not found")
    }
}

function runAction(entrypointId: string, action: GeneratedEntrypointDerivedAction) {
    switch (action.type) {
        case "Command": {
            action.run()

            break;
        }
        case "View": {
            const entrypointName = storedGeneratedEntrypoints[entrypointId]
                .command
                .name

            renderView(entrypointId, entrypointName, action.view)
            break;
        }
    }
}