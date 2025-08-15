import React, { ReactElement } from "react";
import { List } from "@project-gauntlet/api/components";
import { ListOfWindows, openWindows } from "./window/shared";
import { current_os, wayland } from "gauntlet:bridge/internal-all";
import { focusWaylandWindow } from "./window/wayland";
import { focusX11Window } from "./window/x11";

export default function Windows(): ReactElement {
    switch (current_os()) {
        case "linux": {
            if (wayland()) {
                return (
                    <ListOfWindows
                        windows={openWindows()}
                        focusWindow={(windowId) => focusWaylandWindow(windowId)}
                        focusSecond={true}
                    />
                )
            } else {
                return (
                    <ListOfWindows
                        windows={openWindows()}
                        focusWindow={(windowId) => focusX11Window(windowId)}
                        focusSecond={true}
                    />
                )
            }
        }
        default: {
            return (
                <List>
                    <List.Item id="not-supported" title="Not supported on current system"/>
                </List>
            )
        }
    }
}
