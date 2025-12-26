<script lang="ts">
  import { createEventDispatcher, getContext } from "svelte";

  const dispatch = createEventDispatcher();

  const imageName = getContext("imageName")();
  const action = getContext("modalAction")();
  const message = `Are you sure you want to ${action} "${imageName}"?`;
  const actionTitle = action.charAt(0).toUpperCase() + action.slice(1);

  function closeModal(eventName: string) {
    const modal = document.getElementById("confirm-action-backdrop");
    modal.classList.add("closing");

    modal.addEventListener("animationend", () => {
      dispatch(eventName);
    });
  }

  function handleConfirm() {
    closeModal("confirm");
  }

  function handleCancel() {
    closeModal("cancel");
  }

  function handleModalClick(event: CustomEvent) {
    // Stop the event from bubbling up
    event.stopPropagation();
  }
</script>

<div
  class="modal-backdrop"
  id="confirm-action-backdrop"
  onclick={handleCancel}
  >
  <div class="modal-content" onclick={handleModalClick}>
    <div class="inner">
      <!-- svelte-ignore state_referenced_locally -->
      <h2>Confirm {actionTitle}</h2>
      <p>{message}</p>
      <div class="modal-actions">
        <button class="confirm btn" onclick={handleConfirm}>
          Confirm
        </button>
        <button class="btn" onclick={handleCancel}>
          Cancel
        </button>
      </div>
    </div>
  </div>
</div>

<style>
  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
  }

  .confirm {
    border: 1px solid var(--im-warn);
  }

  .confirm:hover:not(:disabled) {
    background: var(--im-warn);
  }

  .confirm:active:not(:disabled) {
    background: var(--im-btn-active-warn);
    border: 1px solid var(--im-btn-active-warn);
  }

  p {
    padding: 12px;
    text-align: center;
    font-weight: 500;
  }
</style>
