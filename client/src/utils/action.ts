import Hammer from "hammerjs";

export function hammerSwipe(node: HTMLElement) {
    const mc = new Hammer(node);

    mc.on("swipeleft swiperight", (ev) => {
        node.dispatchEvent(new CustomEvent("swipe", {
            detail: {
                direction: ev.type,
                timeframe: 300,
                minSwipeDistance: 100
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
