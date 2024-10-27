import { ReactElement } from 'react';
import { Form } from "@project-gauntlet/api/components";

export default function Main(): ReactElement {
    return (
        // docs-code-segment:start
        <Form>
            <Form.PasswordField label={"Password"} value={"burger"}/>
        </Form>
        // docs-code-segment:end
    );
};

