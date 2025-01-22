import { ReactElement } from 'react';
import { Form } from "@project-gauntlet/api/components";

export default function TextFieldExample(): ReactElement {
    return (
        <Form>
            <Form.TextField
                label="The Tragedy of Darth Plagueis the Wise"
                onChange={value => {
                    console.log(`value: ${value}`)
                }}
            />
        </Form>
    );
};

