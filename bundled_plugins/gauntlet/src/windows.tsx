import React, { ReactElement } from "react";
import { List } from "@project-gauntlet/api/components";
import { openWindows } from "./window/shared";
import { current_os, wayland } from "gauntlet:bridge/internal-all";
import { focusWaylandWindow } from "./window/wayland";
import { focusX11Window } from "./window/x11";

export default function Windows(): ReactElement {
    switch (current_os()) {
        case "linux": {
            if (wayland()) {
                return (
                    <ListOfWindows focus={(windowId) => focusWaylandWindow(windowId)}/>
                )
            } else {
                return (
                    <ListOfWindows focus={(windowId) => focusX11Window(windowId)}/>
                )
            }
        }
        default: {
            return (
                <List>
                    <List.Item title="Not supported on current system"/>
                </List>
            )
        }
    }
}

function ListOfWindows({ focus }: { focus: (windowId: string) => void }) {
    return (
        <List>
            {
                Object.entries(openWindows)
                    .map(([_, window]) => (
                            <List.Item key={window.id} title={window.title} onClick={() => { focus(window.id) }}/>
                        )
                    )
            }
        </List>
    )
}