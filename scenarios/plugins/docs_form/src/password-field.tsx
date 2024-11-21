import { ReactElement } from 'react';
import { Form } from "@project-gauntlet/api/components";

export default function Main(): ReactElement {
    return (
        <Form>
            <Form.PasswordField label={"Password"} value={"burger"}/>
        </Form>
    );
};

