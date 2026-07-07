<script lang="ts">
  import { onMount } from "svelte";
  import { fade } from "svelte/transition";
  import IconButton from "@smui/icon-button";
  import type { IconMenuItem } from "../store.ts";
  import { toggleHidden } from "../utils/app.ts";
  import { dropdownPortal } from "../utils/action.ts";

  let { menu, disabled = false } = $props();

  // State to handle open/closed menu
  let isOpen: boolean = $state(false);
  let containerRef = $state();

  // Close dropdown when clicking outside of it
  function handleClickOutside(event: PointerEvent) {
    if (containerRef && !containerRef.contains(event.target)) {
      if (menu.handleClickOutside && isOpen)
        menu.handleClickOutside();
      isOpen = false;
    }
  }

  function menuIconClicked() {
    isOpen = !isOpen;
    if (menu.handleClick) menu.handleClick();
  }

  function menuItemIconClicked(item: IconMenuItem) {
    isOpen = !isOpen;
    if (item.handleClick) item.handleClick();
  }

  onMount(() => {
    document.addEventListener("click", handleClickOutside);
    return () => document.removeEventListener("click", handleClickOutside);
  });

  $effect(() => {
    if (menu.toggleFunc) menu.toggleFunc(isOpen);
  });

  function getTriggerBtn(): HTMLElement | null {
    const el = document.getElementsByClassName("selected")[0] as HTMLElement;
    if (el) return el;

    return null;
  }
</script>

<div bind:this={containerRef}>
  <!-- Main trigger icon button -->
  <IconButton
    title={menu.title}
    class={isOpen ? "material-icons icon-btn selected" : "material-icons icon-btn"}
    onclick={menuIconClicked}
    disabled={disabled}
    >
    {menu.iconName}
  </IconButton>

  <!-- Dropdown menu items -->
  {#if isOpen}
    <div
      id="menu"
      use:dropdownPortal={{ isOpen, trigger: getTriggerBtn() }}
      in:fade={{ duration: 200 }}
      out:fade={{ duration: 200 }}
      >
      {#each menu.items as item (item.title)}
        <div
          class="menu-item"
          onclick={() => menuItemIconClicked(item)}
          >
          <IconButton
            class="material-icons icon-btn"
            >
            {item.iconName}
          </IconButton>
          <span>{item.title}</span>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  #menu {
    position: fixed;
    display: flex;
    flex-direction: column;
    align-items: baseline;
    background: black;
  }

  .menu-item {
    display: block;
    background: none;
    width: 100%;
    white-space: nowrap;
    color: var(--im-text);
    font-size: 12px;
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    padding-right: 5px;
    transition: background 0.2s;
  }

  .menu-item:hover:not(:disabled) {
    background: var(--im-hover-gold);
  }

  .menu-item:active:not(:disabled) {
    background: var(--im-btn-active-gold);
  }

  :global(icon-btn):hover,
  :global(icon-btn):active {
    background: none;
  }
</style>
