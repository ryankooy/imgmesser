<script lang="ts">
  import { getFileExtension, getFileStem } from "../utils/app.ts";

  let { imageName, props, cancel, closeModal } = $props();

  const imageFileStem: string = $derived(getFileStem(imageName));

  let imageCopyFileStem: string = $derived(imageFileStem + "_COPY");
  const imageCopyFileExt: string = $derived(getFileExtension(imageName));

  function handleClick(button: object) {
    closeModal(button.handleClick, getFullFileName());
  }

  function getFullFileName(): string {
    return imageCopyFileStem + "." + imageCopyFileExt;
  }
</script>

<div>
  <div class="save-image-copy-content">
    <div class="form-input">
      <label for="image-copy-input">
        Save image as...
      </label>
      <input
        id="image-copy-input"
        type="text"
        name="image-copy-name"
        bind:value={imageCopyFileStem}
        autofocus
      />
      <div>.{imageCopyFileExt}</div>
    </div>
  </div>

  <div class="modal-actions">
    {#if props.buttons}
      {#each props.buttons as btn (btn.text)}
        <button
          class="btn"
          onclick={() => handleClick(btn)}
          disabled={imageCopyFileStem === imageFileStem}
          >
          {btn.text}
        </button>
      {/each}
    {/if}

    <button class="btn" onclick={cancel}>
      Cancel
    </button>
  </div>
</div>

<style>
  input[name="image-copy-name"] {
    all: unset;
    color: ghostwhite;
    font-style: oblique;
    border-bottom: 1px solid transparent;
    width: 15rem;
    cursor: text;
  }

  input[name="image-copy-name"]:focus {
    border-bottom: 1px solid ghostwhite;
  }

  .form-input {
    display: inline-flex;
    padding: 12px 12px 36px 12px;
    align-items: center;
    gap: 1em;
    flex-wrap: wrap;
    justify-content: center;
  }

  .form-input label {
    width: auto;
    color: var(--im-label);
    font-weight: 600;
    font-size: 14px;
  }

  .save-image-copy-content {
    text-align: center;
  }
</style>
