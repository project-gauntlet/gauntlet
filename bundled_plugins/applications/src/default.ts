interface GeneratedCommand { // TODO Add this type to api
    id: string
    name: string
    fn: () => void
}

export default function Default(): GeneratedCommand[] {
    return [
        {
            id: 'generated-test-1',
            name: 'Application 1',
            fn: () => {
                console.log('opening application 1')
            }
        },
        {
            id: 'generated-test-2',
            name: 'Application 2',
            fn: () => {
                console.log('opening application 2')
            }
        },
        {
            id: 'generated-test-3',
            name: 'Application 3',
            fn: () => {
                console.log('opening application 3')
            }
        },
        {
            id: 'generated-test-4',
            name: 'Application 4',
            fn: () => {
                console.log('opening application 4')
            }
        }
    ]
}
