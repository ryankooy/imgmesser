import Hammer from "hammerjs";

export function hammerSwipe(node: HTMLElement) {
    const mc = new Hammer(node, { touchAction: "none" });

    mc.on("swipeleft swiperight", (ev) => {
        node.dispatchEvent(new CustomEvent("swipe", {
            detail: {
                direction: ev.type,
            }
        }));
    });

    return {
        destroy() {
            // Clean up on component unmount
            mc.destroy();
        }
    };
}
