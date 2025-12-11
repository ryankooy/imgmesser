<script lang="ts">
  import { createEventDispatcher, getContext } from "svelte";

  const dispatch = createEventDispatcher();

  const imageName = getContext("imageName")();
  const action = getContext("modalAction")();
  const message = `Are you sure you want to ${action} "${imageName}"?`;
  const actionTitle = action.charAt(0).toUpperCase() + action.slice(1);

  function handleConfirm() {
    dispatch("confirm");
  }

  function handleCancel() {
    dispatch("cancel");
  }

  function handleModalClick(event: CustomEvent) {
    // Stop the event from bubbling up
    event.stopPropagation();
  }
</script>

<div class="modal-backdrop" onclick={handleCancel}>
  <div
    class="modal-content"
    id="confirm-action"
    onclick={handleModalClick}
    >
    <div class="inner">
      <!-- svelte-ignore state_referenced_locally -->
      <h2>Confirm {actionTitle}</h2>
      <p>{message}</p>
      <div class="modal-actions">
        <button class="confirm" onclick={handleConfirm}>Confirm</button>
        <button onclick={handleCancel}>Cancel</button>
      </div>
    </div>
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.85);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 20px;
    animation: fadeIn 0.2s ease-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .modal-content {
    background: black;
    color: var(--im-text);
    max-width: 600px;
    width: 100%;
    max-height: 90vh;
    overflow-y: auto;
    position: relative;
    padding: 10px;
    animation: slideUp 0.3s ease-out;
  }

  @keyframes slideUp {
    from {
      transform: translateY(20px);
      opacity: 0;
    }
    to {
      transform: translateY(0);
      opacity: 1;
    }
  }

  .inner {
    padding: 20px;
    border: var(--im-border);
  }

  h2 {
    margin: 0 0 24px 0;
    color: var(--im-header-gold);
    font-size: 24px;
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
  }

  button {
    padding: 14px 24px;
    background: none;
    border: var(--im-border);
    color: var(--im-text);
    font-size: 16px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.2s;
  }

  button:hover:not(:disabled) {
    background: var(--im-hover-gold);
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    transform: none;
  }

  .confirm {
    color: var(--im-warn);
    transition: color 0.2s, background 0.2s, border 0.2s;
  }

  .confirm:hover:not(:disabled) {
    color: var(--im-text);
    background: var(--im-warn);
    border: 1px solid var(--im-warn);
  }

  p {
    padding: 12px;
    text-align: center;
    font-weight: 500;
  }
</style>
