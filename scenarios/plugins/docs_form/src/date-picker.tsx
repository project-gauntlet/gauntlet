import { ReactElement } from 'react';
import { Form } from "@project-gauntlet/api/components";

export default function Main(): ReactElement {
    return (
        // docs-code-segment:start
        <Form>
            <Form.DatePicker
                label={"Date"}
                value={"2024-03-22"}
                onChange={value => {
                    console.log(`value: ${value}`)
                }}
            />
        </Form>
        // docs-code-segment:end
    );
};

