import { generatedCommandSearchIndex } from "./command-generator";

// @ts-expect-error does typescript support such symbol declarations?
const denoCore: DenoCore = Deno[Deno.internal].core;
const InternalApi = denoCore.ops;

export async function loadSearchIndex(refreshSearchList: boolean) {
    await InternalApi.load_search_index(generatedCommandSearchIndex(), refreshSearchList);
}