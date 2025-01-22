import { ReactElement } from 'react';
import { Form } from "@project-gauntlet/api/components";

export default function SeparatorExample(): ReactElement {
    return (
        <Form>
            <Form.TextField label="Canon"/>

            <Form.Separator/>

            <Form.TextField label="Legends"/>
        </Form>
    );
};

