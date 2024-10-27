import { ReactElement } from 'react';
import { Form } from "@project-gauntlet/api/components";

export default function Main(): ReactElement {
    return (
        // docs-code-segment:start
        <Form>
            <Form.TextField
                label={"Toppings"}
                onChange={value => {
                    console.log(`value: ${value}`)
                }}
            />
            <Form.Separator/>
            <Form.TextField
                label={"Condiments"}
                onChange={value => {
                    console.log(`value: ${value}`)
                }}
            />
        </Form>
        // docs-code-segment:end
    );
};

