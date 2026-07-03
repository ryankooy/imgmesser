<script lang="ts">
  import { onMount } from "svelte";
  import { fade } from "svelte/transition";
  import IconButton from "@smui/icon-button";
  import type { IconMenuItem } from "../store.ts";
  import { toggleHidden } from "../utils/app.ts";

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
      in:fade={{ duration: 200 }}
      out:fade={{ duration: 200 }}
      >
      {#each menu.items as item (item.title)}
        <IconButton
          title={item.title}
          class="material-icons icon-btn"
          onclick={() => menuItemIconClicked(item)}
          >
          {item.iconName}
        </IconButton>
      {/each}
    </div>
  {/if}
</div>

<style>
  #menu {
    position: absolute;
    background: black;
  }
</style>
