import { ReactElement } from 'react';
import { Form } from "@project-gauntlet/api/components";

export default function PasswordFieldExample(): ReactElement {
    return (
        <Form>
            <Form.PasswordField label="Password" value="K'lor'slug"/>
        </Form>
    );
};

