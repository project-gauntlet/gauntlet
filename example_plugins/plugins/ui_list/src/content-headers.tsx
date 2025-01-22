import { ReactElement } from "react";
import { List } from "@project-gauntlet/api/components";

export default function ContentHeadersExample(): ReactElement {
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
                    <List.Detail.Content.H4>
                        Behind the scenes
                    </List.Detail.Content.H4>
                    <List.Detail.Content.Paragraph>
                        The Ezaraa species first appeared in the canon crossover comic The Screaming Citadel 1, written by Kieron Gillen, illustrated by Marco Checchetto and released on May 10, 2017.
                    </List.Detail.Content.Paragraph>
                </List.Detail.Content>
            </List.Detail>
        </List>
    )
}
