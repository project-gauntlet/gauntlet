import { ReactNode } from 'react';
// @ts-ignore TODO how to add declaration for this?
import { useGauntletContext } from "gauntlet:renderer";

export function useNavigation(): { popView: () => void, pushView: (component: ReactNode) => void } {
    const { popView, pushView }: { popView: () => void, pushView: (component: ReactNode) => void } = useGauntletContext();

    return {
        popView: () => {
            popView()
        },
        pushView: (component: ReactNode) => {
            pushView(component)
        }
    }
}
