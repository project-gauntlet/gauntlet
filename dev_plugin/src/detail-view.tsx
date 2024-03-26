import { ReactElement, useState } from 'react';
import upperCase from "lodash/upperCase";
import { Action, ActionPanel, Detail } from "@project-gauntlet/api/components";
import { useNavigation, useEntrypointPreferences, usePluginPreferences } from "@project-gauntlet/api/hooks";
import { assetDataSync } from "@project-gauntlet/api/helpers";

interface DetailViewEntrypointConfig {
    testBool: boolean
    testEnum: 'item' | 'item_2'
    testListOfStrings: string[]
    testListOfNumbers: number[]
    testNum: number
    testStr: string
}

function TestView(props: { value: number }): ReactElement {
    const { popView } = useNavigation();

    return (
        <Detail>
            <Detail.Content>
                <Detail.Content.Paragraph>
                    Nested view. Value from parent: {props.value}
                </Detail.Content.Paragraph>
            </Detail.Content>
            <Detail.Metadata>
                <Detail.Metadata.TagList label={"test"}>
                    <Detail.Metadata.TagList.Item
                        onClick={() => {
                            popView();
                        }}
                    >
                        Shit Go Back!
                    </Detail.Metadata.TagList.Item>
                </Detail.Metadata.TagList>
            </Detail.Metadata>
        </Detail>
    )
}


export default function DetailView(): ReactElement {
    const [count, setCount] = useState(0);

    const { pushView } = useNavigation();
    const { testBool } = usePluginPreferences<{ testBool: boolean }>();
    const entrypointPreferences = useEntrypointPreferences<DetailViewEntrypointConfig>();

    const env = Deno.env.get("RUST_LOG");
    console.log("RUST_LOG:", env);

    const logoData = assetDataSync("logo.png");

    return (
        <Detail
            actions={
                <ActionPanel title={"action panel"}>
                    <Action
                        title={"action 1"}
                        onAction={() => {
                            console.log("ActionTest 1")
                        }}
                    />
                    <ActionPanel.Section title={"action panel section"}>
                        <Action
                            title={"action 2.1"}
                            onAction={() => {
                                console.log("ActionTest 2.1")
                            }}
                        />
                        <Action
                            id="testAction1"
                            title={"action 2.2"}
                            onAction={() => {
                                console.log("ActionTest 2.2")
                            }}
                        />
                    </ActionPanel.Section>
                    <ActionPanel.Section>
                        <Action
                            id="testAction2"
                            title={"action 3"}
                            onAction={() => {
                                console.log("ActionTest 3")
                            }}
                        />
                    </ActionPanel.Section>
                </ActionPanel>
            }>
            <Detail.Content>
                <Detail.Content.H1>H1 Title</Detail.Content.H1>
                <Detail.Content.H2>H2 Title</Detail.Content.H2>
                <Detail.Content.H3>H3 Title</Detail.Content.H3>
                <Detail.Content.H4>H4 Title</Detail.Content.H4>
                <Detail.Content.H5>H5 Title</Detail.Content.H5>
                <Detail.Content.H6>H6 Title</Detail.Content.H6>
                <Detail.Content.Image source={{ data: logoData }}/>
                <Detail.Content.Link href={"https://google.com/"}>Google Link</Detail.Content.Link>
                <Detail.Content.CodeBlock>Code block Test</Detail.Content.CodeBlock>
                <Detail.Content.HorizontalBreak/>
                <Detail.Content.Paragraph>
                    You clicked {count} times
                </Detail.Content.Paragraph>
                <Detail.Content.Paragraph>
                    Plugin config: {JSON.stringify(testBool)}
                </Detail.Content.Paragraph>
                <Detail.Content.Paragraph>
                    Entrypoint config: {JSON.stringify(entrypointPreferences)}
                </Detail.Content.Paragraph>
                <Detail.Content.H4>Another H4 Title</Detail.Content.H4>
                <Detail.Content.Paragraph>
                    Lorem ipsum {upperCase("dolor sit amet")}, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore
                    et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut
                    aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse
                    cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in
                    culpa qui officia deserunt mollit anim id est laborum.
                </Detail.Content.Paragraph>
            </Detail.Content>
            <Detail.Metadata>
                <Detail.Metadata.TagList label="Tags 1">
                    <Detail.Metadata.TagList.Item
                        onClick={() => {
                            console.log("test " + upperCase("events") + count)
                            setCount(count + 1);
                        }}
                    >
                        Tag
                    </Detail.Metadata.TagList.Item>
                    <Detail.Metadata.TagList.Item onClick={() => {
                        pushView(<TestView value={1}/>)
                    }}>
                        Push New View
                    </Detail.Metadata.TagList.Item>
                </Detail.Metadata.TagList>
                <Detail.Metadata.Separator/>
                <Detail.Metadata.Link label="Test 2" href={""}>
                    Link text
                </Detail.Metadata.Link>
                <Detail.Metadata.Value label="Label 3">
                    Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do
                    eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis
                    nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute
                    irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.
                    Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit
                    anim id est laborum.
                </Detail.Metadata.Value>
                <Detail.Metadata.Icon label="Label 4" icon="icon"/>
                <Detail.Metadata.Value label="Label 5">
                    Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do
                    eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis
                    nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute
                    irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.
                    Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit
                    anim id est laborum.
                </Detail.Metadata.Value>
            </Detail.Metadata>
        </Detail>
    );
};

