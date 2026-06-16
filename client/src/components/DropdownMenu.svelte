<script lang="ts">
  import { onMount } from "svelte";
  import IconButton from "@smui/icon-button";

  let { imageDataUrl, menu } = $props();

  // State to handle open/closed menu
  let isOpen: boolean = $state(false);
  let containerRef = $state();

  // Close dropdown when clicking outside of it
  function handleClickOutside(event) {
    if (containerRef && !containerRef.contains(event.target))
      isOpen = false;
  }

  onMount(() => {
    document.addEventListener("click", handleClickOutside);
    return () => document.removeEventListener("click", handleClickOutside);
  });
</script>

<div class="relative inline-block text-left" bind:this={containerRef}>
  <!-- Main trigger icon button -->
  <IconButton
    title={menu.title}
    class={isOpen ? "material-icons icon-btn selected" : "material-icons icon-btn"}
    onclick={() => isOpen = !isOpen}
    disabled={!imageDataUrl}
    >
    {menu.iconName}
  </IconButton>

  <!-- Dropdown menu items -->
  {#if isOpen}
    <div class="menu">
      {#each menu.items as item (item.title)}
        <IconButton
          title={item.title}
          class="material-icons icon-btn"
          onclick={item.func}
          >
          {item.iconName}
        </IconButton>
      {/each}
    </div>
  {/if}
</div>

<style>
  .menu {
    position: absolute;
  }
</style>
