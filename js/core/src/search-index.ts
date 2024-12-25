import { generatedCommandSearchIndex } from "./entrypoint-generator";
import { reload_search_index } from "ext:core/ops";

export async function reloadSearchIndex(refreshSearchList: boolean) {
    await reload_search_index(generatedCommandSearchIndex(), refreshSearchList);
}