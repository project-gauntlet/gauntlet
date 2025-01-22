import { ReactElement } from 'react';
import { Form } from "@project-gauntlet/api/components";

export default function CheckboxExample(): ReactElement {
    return (
        <Form>
            <Form.Checkbox
                label="Have heard about"
                title="The Tragedy of Darth Plagueis the Wise"
                value={true}
                onChange={value => {
                    console.log(`value: ${value}`)
                }}
            />
        </Form>
    );
};

