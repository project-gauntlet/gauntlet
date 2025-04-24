import { ReactElement } from "react";
import { Action, ActionPanel, Detail, List } from "@project-gauntlet/api/components";
import { useNavigation } from "@project-gauntlet/api/hooks";

function SubView(): ReactElement {
    const { popView } = useNavigation();

    return (
        <List actions={
            <ActionPanel>
                <Action
                    label={"Open subview"}
                    onAction={id => {
                        switch (id) {
                            case "run": {
                                console.log("Running the Gauntlet...")
                                break;
                            }
                            case "pop": {
                                popView()
                                break;
                            }
                        }
                    }}
                />
            </ActionPanel>
        }>
            <List.Item id={"run"} title={"Run"}/>
            <List.Item id={"pop"} title={"Go Back"}/>
        </List>
    )
}

export default function View(): ReactElement {
    const { pushView } = useNavigation();

    return (
        <List actions={
            <ActionPanel>
                <Action
                    label={"Open subview"}
                    onAction={id => {
                        if (id == "push") {
                            pushView(<SubView/>)
                        }
                    }}
               />
            </ActionPanel>
        }>
            <List.Item id={"push"} title={"Open subview"}/>
        </List>
    )
}
