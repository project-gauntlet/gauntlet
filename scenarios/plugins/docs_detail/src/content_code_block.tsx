import { Detail } from "@project-gauntlet/api/components";
import { ReactNode } from "react";


const code = `
    fib :: Integer -> Integer
    fib 0 = 0
    fib 1 = 1
    fib n = fib (n-1) + fib (n-2)
`.trimStart().replace(/^ {4}/g, '');

export default function Main(): ReactNode {
    return (
        // docs-code-segment:start
        <Detail>
            <Detail.Content>
                <Detail.Content.CodeBlock>
                    {code}
                </Detail.Content.CodeBlock>
            </Detail.Content>
        </Detail>
        // docs-code-segment:end
    )
}
