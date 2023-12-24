import { ReactElement } from 'react';
import { Form } from "@project-gauntlet/api/components";

export default function DetailView(): ReactElement {
    return (
        <Form>
            <Form.TextField/>
            <Form.PasswordField/>
            <Form.Separator/>
            <Form.Checkbox/>
            <Form.Select/>
            <Form.DatePicker value={"2024-03-22"} onChange={(value) => {
                console.log(`value: ${value}`)
            }}/>
        </Form>
    );
};

