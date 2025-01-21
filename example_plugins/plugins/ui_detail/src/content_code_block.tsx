import { Detail } from "@project-gauntlet/api/components";
import { ReactNode } from "react";

const code = `\
fib :: Integer -> Integer
fib 0 = 0
fib 1 = 1
fib n = fib (n-1) + fib (n-2)
`

export default function Main(): ReactNode {
    return (
        <Detail>
            <Detail.Content>
                <Detail.Content.CodeBlock>
                    {code}
                </Detail.Content.CodeBlock>
            </Detail.Content>
        </Detail>
    )
}
