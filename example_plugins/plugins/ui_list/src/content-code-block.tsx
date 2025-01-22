import { ReactElement } from "react";
import { List } from "@project-gauntlet/api/components";

export default function ContentCodeBlockExample(): ReactElement {
    return (
        <List>
            <List.Item id="ezaraa" title="Ezaraa"/>
            <List.Detail>
                <List.Detail.Content>
                    <List.Detail.Content.CodeBlock>
                        Hah! Why pay when one can slay? Then retrieve the bauble from its smoking chassis! It is the Ezaraa way!"
                        "For the glory of the Ezaraa Dominion!"
                        [The Ezaraas take a single causality]
                        "Retreat!"
                    </List.Detail.Content.CodeBlock>
                    <List.Detail.Content.Paragraph>
                        ―Ezaraa warriors during the Rur Crystal incident
                    </List.Detail.Content.Paragraph>
                </List.Detail.Content>
            </List.Detail>
        </List>
    )
}
