import { Detail } from "@project-gauntlet/api/components";
import { ReactNode } from "react";

export default function Main(): ReactNode {
    return (
        <Detail>
            <Detail.Content>
                <Detail.Content.Paragraph>
                    Did you know…
                </Detail.Content.Paragraph>
                <Detail.Content.HorizontalBreak/>
                <Detail.Content.Paragraph>
                    …that the Vandrayk Scale was a measurement that related likelihood for an asteroid to house the exogorth species?
                </Detail.Content.Paragraph>
            </Detail.Content>
        </Detail>
    )
}
