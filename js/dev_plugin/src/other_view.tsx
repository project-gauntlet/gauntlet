import lowerCase from "lodash/lowerCase";
import { Box } from "placeholdername-component-model";
import { ReactElement } from "react";

export default function View(): ReactElement {
    return (
        <Box test={1}>
            test {lowerCase("events")}
        </Box>
    );
};

