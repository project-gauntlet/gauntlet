import { ReactNode } from 'react';
// @ts-ignore TODO how to add declaration for this?
import { useGauntletContext } from "gauntlet:renderer";

export function useNavigation(): { pop: () => void, push: (component: ReactNode) => void } {
    const { popView, pushView }: { popView: () => void, pushView: (component: ReactNode) => void } = useGauntletContext();

    return {
        pop: () => {
            popView()
        },
        push: (component: ReactNode) => {
            pushView(component)
        }
    }
}