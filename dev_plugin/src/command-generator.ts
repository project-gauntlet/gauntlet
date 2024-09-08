import { GeneratedCommand } from "@project-gauntlet/api/helpers";

export default function CommandGenerator(): GeneratedCommand[] {
    return [
        {
            id: 'generated-test-1',
            name: 'Generated Item 1',
            fn: () => {
                new Promise(() => {
                    throw new Error("gen")
                })

                console.log('generated-test-1')
            }
        },
        {
            id: 'generated-test-2',
            name: 'Generated Item 2',
            fn: () => {
                console.log('generated-test-2')
            }
        },
        {
            id: 'generated-test-3',
            name: 'Generated Item 3',
            fn: () => {
                console.log('generated-test-3')
            }
        },
        {
            id: 'generated-test-4',
            name: 'Generated Item 4',
            fn: () => {
                console.log('generated-test-4')
            }
        }
    ]
}
