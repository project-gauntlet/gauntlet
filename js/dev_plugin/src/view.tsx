import { ReactElement, useState } from 'react';
import upperCase from "lodash/upperCase";
import { Box } from "placeholdername-component-model";

export default function View(): ReactElement {

    const [count, setCount] = useState(0);

    return (
        <Box test={1}>
            test
            <Box.Text>
                {count}
            </Box.Text>
            <Box.Text>
                You clicked
                {count}
                times
            </Box.Text>
            <Box.Button onClick={() => {
                console.log("test " + upperCase("events") + count)
                setCount(count + 1);
            }}>
                Click me
            </Box.Button>
            <Box.Text>Test</Box.Text>
            <Box.Text>Test</Box.Text>
        </Box>
    );
};

