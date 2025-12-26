<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import IconButton from "@smui/icon-button";

  const { message = null } = $props();

  const dispatch = createEventDispatcher();

  function close() {
    const modal = document.getElementById("alert-backdrop");
    modal.classList.add("closing");

    modal.addEventListener("animationend", () => {
      dispatch("close");
    });
  }

  function handleModalClick(event: CustomEvent) {
    // Stop the event from bubbling up
    event.stopPropagation();
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      close();
    }
  }

  setTimeout(() => {
    close();
  }, 5000);
</script>

<div
  class="modal-backdrop"
  id="alert-backdrop"
  onclick={handleBackdropClick}
  >
  <div
    class="modal-content alert-content"
    onclick={handleModalClick}
    >
    <IconButton
      class="material-icons icon-btn close-btn no-bkg"
      onclick={close}
      aria-label="Close"
      >
      close
    </IconButton>

    <p>{message}</p>
  </div>
</div>

<style>
  .alert-content {
    background: var(--im-warn);
  }

  p {
    padding: 12px;
    color: var(--im-text);
    text-align: center;
    font-weight: 500;
    background: var(--im-warn);
  }
</style>
