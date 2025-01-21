import { ReactElement } from "react";
import { Icons, List } from "@project-gauntlet/api/components";

export default function Main(): ReactElement {
    return (
        <List>
            <List.Item id="aruzan" title="Aruzan"/>
            <List.Detail>
                <List.Detail.Metadata>
                    <List.Detail.Metadata.TagList label="Appearances">
                        <List.Detail.Metadata.TagList.Item>Bounty Hunters 14</List.Detail.Metadata.TagList.Item>
                        <List.Detail.Metadata.TagList.Item>Bounty Hunters 20</List.Detail.Metadata.TagList.Item>
                        <List.Detail.Metadata.TagList.Item>Bounty Hunters 23</List.Detail.Metadata.TagList.Item>
                        <List.Detail.Metadata.TagList.Item>Bounty Hunters 24</List.Detail.Metadata.TagList.Item>
                        <List.Detail.Metadata.TagList.Item>Bounty Hunters 35</List.Detail.Metadata.TagList.Item>
                        <List.Detail.Metadata.TagList.Item>"Tall Tales" — Revelations (2023) 1</List.Detail.Metadata.TagList.Item>
                        <List.Detail.Metadata.TagList.Item>Bounty Hunters 42</List.Detail.Metadata.TagList.Item>
                    </List.Detail.Metadata.TagList>
                </List.Detail.Metadata>
            </List.Detail>
        </List>
    )
}
