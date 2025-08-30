import React, { ReactElement } from "react";
import { List } from "@project-gauntlet/api/components";
import { ListOfWindows, openWindows } from "./window/shared";
import { current_os, wayland } from "gauntlet:bridge/internal-all";
import { macos_focus_window } from "gauntlet:bridge/internal-macos";
import { linux_wayland_focus_window, linux_x11_focus_window } from "gauntlet:bridge/internal-linux";

export default function Windows(): ReactElement {
    switch (current_os()) {
        case "linux": {
            if (wayland()) {
                return (
                    <ListOfWindows
                        windows={openWindows()}
                        focusWindow={linux_wayland_focus_window}
                        focusSecond={true}
                    />
                )
            } else {
                return (
                    <ListOfWindows
                        windows={openWindows()}
                        focusWindow={linux_x11_focus_window}
                        focusSecond={true}
                    />
                )
            }
        }
        case "macos": {
            return (
                <ListOfWindows
                    windows={openWindows()}
                    focusWindow={macos_focus_window}
                    focusSecond={true}
                />
            )
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
