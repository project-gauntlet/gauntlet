import { ReactElement } from "react";
import { List } from "@project-gauntlet/api/components";

export default function ContentParagraphExample(): ReactElement {
    return (
        <List>
            <List.Item id="adarian" title="Adarian"/>
            <List.Item id="aruzan" title="Aruzan"/>
            <List.Item id="blutopian" title="Blutopian"/>
            <List.Item id="caphex" title="Caphex"/>
            <List.Item id="condluran" title="Condluran"/>
            <List.Item id="frozian" title="Frozian"/>
            <List.Item id="evereni" title="Evereni"/>
            <List.Item id="ezaraa" title="Ezaraa"/>
            <List.Item id="houk" title="Houk"/>
            <List.Item id="inleshat" title="Inleshat"/>
            <List.Detail>
                <List.Detail.Content>
                    <List.Detail.Content.Paragraph>
                        Hah! Why pay when one can slay? Then retrieve the bauble from its smoking chassis! It is the Ezaraa way!"
                        "For the glory of the Ezaraa Dominion!"
                        [The Ezaraas take a single causality]
                        "Retreat!"
                    </List.Detail.Content.Paragraph>
                    <List.Detail.Content.Paragraph>
                        ―Ezaraa warriors during the Rur Crystal incident
                    </List.Detail.Content.Paragraph>
                </List.Detail.Content>
            </List.Detail>
        </List>
    )
}
