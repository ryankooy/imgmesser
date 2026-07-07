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

export function dropdownPortal(node: HTMLElement, { isOpen, trigger }) {
    const positionDropdown = () => {
        if (!isOpen || !trigger) return;

        const triggerRect = trigger.getBoundingClientRect();
        const dropdownRect = node.getBoundingClientRect();

        // Default to opening below and aligned left
        let top = triggerRect.bottom + 8;
        let left = triggerRect.left;

        // Flip vertically if it goes off screen bottom
        if (top + dropdownRect.height > window.innerHeight) {
            top = triggerRect.top - dropdownRect.height - 8;
        }

        // Flip horizontally if it goes off screen right
        if (left + dropdownRect.width > window.innerWidth) {
            left = triggerRect.right - dropdownRect.width;
        }

        node.style.position = "fixed";
        node.style.top = `${top}px`;
        node.style.left = `${left}px`;
        node.style.zIndex = "9999";
    };

    if (isOpen) {
        positionDropdown();
        window.addEventListener("resize", positionDropdown);
        window.addEventListener("scroll", positionDropdown, true);
    }

    return {
        update(params) {
            isOpen = params.isOpen;
            trigger = params.trigger;
            positionDropdown();
        },
        destroy() {
            window.removeEventListener("resize", positionDropdown);
            window.removeEventListener("scroll", positionDropdown, true);
        }
    };
}
