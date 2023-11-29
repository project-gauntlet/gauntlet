import { ReactElement } from 'react';
import upperCase from "lodash/upperCase";
import { Detail } from "placeholdername-component-model";

export default function View(): ReactElement {

    return (
        <Detail>
            <Detail.Content>
                <Detail.Metadata.Separator/>
                <Detail.Content.H1>Title</Detail.Content.H1>
                <Detail.Content.Text>
                    You clicked
                    {true}
                    {false}
                    {1}
                    {["test", false, undefined,["test3", 5], null, 3]}
                    {undefined}
                    {null}
                    {upperCase("times")}
                </Detail.Content.Text>
            </Detail.Content>
        </Detail>
    );
};

