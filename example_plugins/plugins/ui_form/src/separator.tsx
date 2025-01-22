import { ReactElement } from 'react';
import { Form } from "@project-gauntlet/api/components";

export default function SeparatorExample(): ReactElement {
    return (
        <Form>
            <Form.TextField
                label="Toppings"
                onChange={value => {
                    console.log(`value: ${value}`)
                }}
            />
            <Form.Separator/>
            <Form.TextField
                label="Condiments"
                onChange={value => {
                    console.log(`value: ${value}`)
                }}
            />
        </Form>
    );
};

