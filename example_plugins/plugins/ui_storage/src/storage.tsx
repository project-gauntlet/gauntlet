import { ReactElement, useState } from "react";
import { ActionPanel, List } from "@project-gauntlet/api/components";
import { useCache, useStorage } from "@project-gauntlet/api/hooks";

export default function View(): ReactElement {
    const [state, setState] = useState<number>(0);
    const [cache, setCache] = useCache<number>("cache-item-key", 0);
    const [storage, setStorage] = useStorage<number>("storage-item-key", 0);
    
    return (
        <List actions={
            <ActionPanel>
                <ActionPanel.Action
                    label={"Bump"}
                    onAction={id => {
                        switch (id) {
                            case "state": {
                                setState(prevState => prevState + 1);
                                break
                            }
                            case "cache": {
                                setCache(prevState => prevState + 1);
                                break
                            }
                            case "storage": {
                                setStorage(prevState => prevState + 1);
                                break
                            }
                        }
                    }}
                />
            </ActionPanel>
        }>
            <List.Item id="state" title={"This value will reset after reopening view - " + state}/>
            <List.Item id="cache" title={"This value will reset after restarting plugin - " + cache}/>
            <List.Item id="storage" title={"This value will persist between Gauntlet restarts - " + storage}/>
        </List>
    )
}
