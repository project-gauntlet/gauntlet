import { ReactElement } from 'react';
import { Form } from "@project-gauntlet/api/components";

export default function CheckboxExample(): ReactElement {
    return (
        <Form>
            <Form.Checkbox
                label="Cheese"
                value={true}
                onChange={value => {
                    console.log(`value: ${value}`)
                }}
            />
        </Form>
    );
};

