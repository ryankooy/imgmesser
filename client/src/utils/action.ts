import Hammer from "hammerjs";

export function hammerSwipe(node: HTMLElement) {
    // Return early if device doesn't support touch
    if (!("ontouchstart" in window || navigator.maxTouchPoints > 0)) return;

    const mc = new Hammer(node, { touchAction: "none" });

    mc.on("swipeleft swiperight", (ev) => {
        // Don't dispatch swipe event if element has "panning" class
        if (!node.classList.contains("panning")) {
            node.dispatchEvent(new CustomEvent("swipe", {
                detail: {
                    direction: ev.type,
                }
            }));
        }
    });

    return {
        destroy() {
            // Clean up on component unmount
            mc.destroy();
        }
    };
}
