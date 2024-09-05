import { generatedCommandSearchIndex } from "./command-generator";

// @ts-expect-error does typescript support such symbol declarations?
const denoCore: DenoCore = Deno[Deno.internal].core;
const InternalApi = denoCore.ops;

export async function reloadSearchIndex(refreshSearchList: boolean) {
    await InternalApi.reload_search_index(generatedCommandSearchIndex(), refreshSearchList);
}