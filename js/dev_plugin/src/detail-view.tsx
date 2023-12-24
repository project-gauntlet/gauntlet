import { ReactElement, useState } from 'react';
import upperCase from "lodash/upperCase";
import { Detail } from "@project-gauntlet/api/components";
// import { useSomething } from "@project-gauntlet/api/hooks";

export default function DetailView(): ReactElement {
    const [count, setCount] = useState(0);

    return (
        <Detail>
            <Detail.Content>
                <Detail.Content.H1>H1 Title</Detail.Content.H1>
                <Detail.Content.H2>H2 Title</Detail.Content.H2>
                <Detail.Content.H3>H3 Title</Detail.Content.H3>
                <Detail.Content.H4>H4 Title</Detail.Content.H4>
                <Detail.Content.H5>H5 Title</Detail.Content.H5>
                <Detail.Content.H6>H6 Title</Detail.Content.H6>
                <Detail.Content.Image/>
                <Detail.Content.Link href={"https://google.com/"}>Google Link</Detail.Content.Link>
                <Detail.Content.CodeBlock>Code block Test</Detail.Content.CodeBlock>
                <Detail.Content.HorizontalBreak/>
                <Detail.Content.Paragraph>
                    You clicked {count} times
                    {true}
                    {false}
                    {count}
                    {["test", false, undefined, ["test3", 5], null, 3]}
                    {undefined}
                    {null}
                    {upperCase("times")}
                </Detail.Content.Paragraph>
                <Detail.Content.H4>Another H4 Title</Detail.Content.H4>
                <Detail.Content.Paragraph>
                    Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore
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
                    <Detail.Metadata.TagList.Item>
                        Another Tag
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

