<script lang="ts">
  import { getContext } from "svelte";
  import { ModalType } from "../store.ts";

  let { props, onCancel } = $props();

  const imageName: string = getContext("imageName")();
  let imageCopyName: string = $state(imageName);

  function closeModal(func: (() => void) | null) {
    const modal = document.getElementById("confirm-action-backdrop");
    modal.classList.add("closing");

    modal.addEventListener("animationend", () => {
      if (func) func();
    });
  }

  function handleConfirm() {
    if (props.options) closeModal(props.options.handleAction);
  }

  function handleCancel() {
    closeModal(onCancel);
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
      <h2>{props.title}</h2>

      {#if props.type === ModalType.Confirm}
        <p>
          {#if props.options}
            Are you sure you want to {props.options.actionText} image <em>{imageName}</em>?
            {#if props.options.extraText}
              {props.options.extraText}
            {/if}
          {:else}
            Are you sure?
          {/if}
        </p>
      {:else if props.type === ModalType.SaveImageCopy}
        <div class="form-input">
          <label for="image-copy-input">
            Save image as...
          </label>
          <input
            id="image-copy-input"
            type="text"
            name="image-copy-name"
            bind:value={imageCopyName}
            autofocus
          />
        </div>
      {:else if props.text}
        <p>{props.text}</p>
      {/if}

      <div class="modal-actions">
        {#if props.type === ModalType.Confirm}
          <button class="confirm btn" onclick={handleConfirm}>
            Confirm
          </button>
        {:else if !!props.buttons}
          {#each props.buttons as btn (btn.text)}
            <button class="btn" onclick={btn.handleClick}>
              {btn.text}
            </button>
          {/each}
        {/if}

        <button class="btn" onclick={handleCancel}>
          {#if props.type === ModalType.Confirm || !!props.buttons}
            Cancel
          {:else}
            OK
          {/if}
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

  input[name="image-copy-name"] {
    all: unset;
    color: ghostwhite;
    font-style: oblique;
    border-bottom: 1px solid transparent;
    width: 10rem;
    cursor: text;
  }

  input[name="image-copy-name"]:focus {
    border-bottom: 1px solid ghostwhite;
  }

  .form-input {
    padding: 12px 12px 36px 12px;
    text-align: center;
  }

  .form-input label {
    width: auto;
    color: var(--im-label);
    font-size: 14px;
    margin-right: 2em;
  }

  p {
    padding: 12px;
    text-align: center;
    font-weight: 500;
  }
</style>
