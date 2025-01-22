import { ReactElement } from 'react';
import { Form } from "@project-gauntlet/api/components";

export default function TextFieldExample(): ReactElement {
    return (
        <Form>
            <Form.TextField
                label="Homeworld"
                onChange={value => {
                    console.log(`homeworld: ${value}`)
                }}
            />
        </Form>
    );
};

