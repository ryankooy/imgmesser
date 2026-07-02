<script lang="ts">
  import { getContext } from "svelte";
  import { ModalType } from "../store.ts";
  import { getFileExtension, getFileStem } from "../utils/app.ts";
  import SaveImageCopyModal from "./SaveImageCopyModal.svelte";

  let { props, onCancel } = $props();

  const imageName: string = getContext("imageName")();
  let imageCopyFileStem: string = $state(getFileStem(imageName));
  let imageCopyFileExt: string = $state(getFileExtension(imageName));

  function handleCloseModal(fn: (() => void) | ((text: string) => void) | null, value: string | null) {
    const modal = document.getElementById("confirm-action-backdrop");
    modal.classList.add("closing");

    if (fn) {
      modal.addEventListener("animationend", () => {
        if (value != null)
          fn(value);
        else
          fn();
      });
    }
  }

  function handleConfirm() {
    if (props.options) handleCloseModal(props.options.handleAction);
  }

  function handleCancel() {
    handleCloseModal(onCancel);
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

        <div class="modal-actions">
          <button class="confirm btn" onclick={handleConfirm}>
            Confirm
          </button>
          <button class="btn" onclick={handleCancel}>
            Cancel
          </button>
        </div>
      {:else if props.type === ModalType.SaveImageCopy}
        <SaveImageCopyModal
          imageName={imageName}
          props={props}
          cancel={handleCancel}
          closeModal={handleCloseModal}
        />
      {:else}
        {#if props.text}
          <p>{props.text}</p>
        {/if}

        <div class="modal-actions">
          <button class="btn" onclick={handleCancel}>
            OK
          </button>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  p {
    padding: 12px;
    text-align: center;
    font-weight: 500;
  }
</style>
