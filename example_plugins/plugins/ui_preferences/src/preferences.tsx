import { ReactElement } from "react";
import { List } from "@project-gauntlet/api/components";
import { useEntrypointPreferences, usePluginPreferences } from "@project-gauntlet/api/hooks";

interface PluginPreferences { 
    testBool: boolean
}

interface EntrypointPreference {
    testStr: string 
}

export default function View(): ReactElement {
    const pluginPreferences = usePluginPreferences<PluginPreferences>();
    const entrypointPreferences = useEntrypointPreferences<EntrypointPreference>();
    
    return (
        <List>
            <List.Item id="item" title={"Plugin - " + pluginPreferences.testBool}/>
            <List.Item id="item" title={"Entrypoint - " + entrypointPreferences.testStr}/>
        </List>
    )
}
