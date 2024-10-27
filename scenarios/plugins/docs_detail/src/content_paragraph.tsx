import { Detail } from "@project-gauntlet/api/components";
import { ReactNode } from "react";

export default function Main(): ReactNode {
    return (
        // docs-code-segment:start
        <Detail>
            <Detail.Content>
                <Detail.Content.Paragraph>
                    The Ezaraa were a species of warmongering carnivorous sentients that were native to the the planet Ezaraa.
                    They intended to overthrow the Galactic Empire, only to replace it with their own dominion and feed on the other species, which they deemed as lesser to them.
                    To arm their revolution, the dominion sent Ezaraa to take advantage of opportunities such as the Auction of Rur.
                </Detail.Content.Paragraph>
            </Detail.Content>
        </Detail>
        // docs-code-segment:end
    )
}
