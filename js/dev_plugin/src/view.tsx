import { ReactElement, useState } from 'react';
import upperCase from "lodash/upperCase";
import { Detail } from "@project-gauntlet/api/components";
// import { useSomething } from "@project-gauntlet/api/hooks";

export default function View(): ReactElement {
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
                <Detail.Content.Code></Detail.Content.Code>
                <Detail.Content.Image></Detail.Content.Image>
                <Detail.Content.Link href={""}>s</Detail.Content.Link>
                <Detail.Content.CodeBlock></Detail.Content.CodeBlock>
                <Detail.Content.HorizontalBreak/>
                <Detail.Content.Text>
                    You clicked
                    {true}
                    {false}
                    {count}
                    {["test", false, undefined, ["test3", 5], null, 3]}
                    {undefined}
                    {null}
                    {upperCase("times")}
                </Detail.Content.Text>
            </Detail.Content>
            <Detail.Metadata>
                <Detail.Metadata.Item>
                    <Detail.Metadata.Item.Text>Test item text</Detail.Metadata.Item.Text>
                    <Detail.Metadata.Item.Tag
                        onClick={() => {
                            console.log("test " + upperCase("events") + count)
                            setCount(count + 1);
                        }}
                    >
                        Tag
                    </Detail.Metadata.Item.Tag>
                </Detail.Metadata.Item>
                <Detail.Metadata.Separator/>
                <Detail.Metadata.Item>
                    <Detail.Metadata.Item.Text>Test metadata 1</Detail.Metadata.Item.Text>
                    <Detail.Metadata.Item.Link href={""}>Test Link</Detail.Metadata.Item.Link>
                    <Detail.Metadata.Item.Text>Test metadata 2</Detail.Metadata.Item.Text>
                </Detail.Metadata.Item>
            </Detail.Metadata>
        </Detail>
    );
};

