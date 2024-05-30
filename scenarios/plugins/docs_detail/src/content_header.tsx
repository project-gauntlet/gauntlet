import { Detail } from "@project-gauntlet/api/components";
import { ReactNode } from "react";

export default function Main(): ReactNode {
    return (
        <Detail>
            <Detail.Content>
                <Detail.Content.H1>
                    Header H1
                </Detail.Content.H1>
                <Detail.Content.H2>
                    Header H2
                </Detail.Content.H2>
                <Detail.Content.H3>
                    Header H3
                </Detail.Content.H3>
                <Detail.Content.H4>
                    Header H4
                </Detail.Content.H4>
                <Detail.Content.H5>
                    Header H5
                </Detail.Content.H5>
                <Detail.Content.H6>
                    Header H6
                </Detail.Content.H6>
            </Detail.Content>
        </Detail>
    )
}
