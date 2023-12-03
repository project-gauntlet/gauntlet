import { ReactElement, useState } from 'react';
import upperCase from "lodash/upperCase";
import { Detail } from "@project-gauntlet/api/components";
// import { useSomething } from "@project-gauntlet/api/hooks";

export default function View(): ReactElement {
    const [count, setCount] = useState(0);

    return (
        <Detail>
            <Detail.Content>
                <Detail.Metadata.Separator/>
                <Detail.Content.H1>Title</Detail.Content.H1>
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
                    <Detail.Metadata.Item.Text>Test</Detail.Metadata.Item.Text>
                    <Detail.Metadata.Item.Tag
                        onClick={() => {
                            console.log("test " + upperCase("events") + count)
                            setCount(count + 1);
                        }}
                    >
                        Tag
                    </Detail.Metadata.Item.Tag>
                </Detail.Metadata.Item>
            </Detail.Metadata>
        </Detail>
    );
};

