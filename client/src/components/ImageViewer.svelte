<script lang="ts">
  import { createEventDispatcher, onMount, setContext } from "svelte";
  import IconButton from "@smui/icon-button";
  import AlertModal from "./AlertModal.svelte";
  import ConfirmModal from "./ConfirmModal.svelte";
  import type { ImageMeta } from "../store.ts";
  import { getImageDataUrl, imageUrl } from "../utils/api.ts";
  import {
    formatDate, formatFileSize, formatImageType,
    getFileExtension, getFileStem,
  } from "../utils/app.ts";

  const dispatch = createEventDispatcher();

  let { gallery = null, image = null, imageIds = [] } = $props();

  const meta: ImageMeta | null = $derived(image.meta ?? null);
  let imageDataUrl: string = $derived(image.url ?? "");

  const multiVersion: boolean = $derived(meta.version_count > 1);

  let loading: boolean = $state(false);
  let editing: boolean = $state(false);
  let showConfirmDeleteModal: boolean = $state(false);
  let showAlertModal: boolean = $state(false);
  let nextPageExists: boolean = $state(false);
  let prevPageExists: boolean = $state(false);

  let alertText: string | null = $state(null);

  const imageName: string = $derived(meta.name);
  setContext("imageName", () => imageName);

  const modalAction: string = "delete";
  setContext("modalAction", () => modalAction);

  let editableFileStem: string = $derived(getFileStem(imageName));

  onMount(() => {
    loadImageData();
  });

  $effect(() => {
    if (gallery) {
      nextPageExists = gallery.more;
      prevPageExists = gallery.current > 1;
    }
  });

  async function loadImageData() {
    loading = true;
    const dataUrl = await getImageDataUrl(image.id);

    if (dataUrl) {
      dispatch("selectDataUrl", dataUrl);
      imageDataUrl = dataUrl;
    }

    loading = false;
  }

  async function deleteImage() {
    try {
      const response = await fetch(`${imageUrl(image.id)}/delete`, {
        method: "POST",
      });

      if (response.ok) {
        dispatch("imageUpdate");
      } else {
        setAlertMessage("Failed to delete image");
      }
    } catch (err) {
      console.error("Error fetching:", err);
    }
  }

  async function renameImage() {
    editing = false;

    const newImageName = getNewImageFileName();
    if (newImageName === imageName) return;

    try {
      const response = await fetch(`${imageUrl(image.id)}/rename`, {
        method: "POST",
        headers: {"Content-Type": "application/json"},
        body: JSON.stringify({ image_name: newImageName }),
      });

      const data = await response.json();

      if (response.ok) {
        if (data.updated) {
          await handleUpdatedImage();
        }
      } else {
        if (data.error && data.error.includes("duplicate")) {
          setAlertMessage("An image with that name already exists");
        } else {
          setAlertMessage("Failed to rename image");
        }
      }
    } catch (err) {
      console.error("Error fetching:", err);
    }
  }

  async function revertImage() {
    try {
      const response = await fetch(`${imageUrl(image.id)}/revert`, {
        method: "POST",
      });

      if (response.ok) {
        const data = await response.json();
        if (data.updated) {
          await handleUpdatedImage();
        }
      } else {
        setAlertMessage("Failed to revert image");
      }
    } catch (err) {
      console.error("Error fetching:", err);
    }
  }

  async function restoreImage() {
    try {
      const response = await fetch(`${imageUrl(image.id)}/restore`, {
        method: "POST",
      });

      if (response.ok) {
        const data = await response.json();
        if (data.updated) {
          await handleUpdatedImage();
        }
      } else {
        setAlertMessage("Failed to restore image");
      }
    } catch (err) {
      console.error("Error fetching:", err);
    }
  }

  async function handleUpdatedImage() {
    dispatch("imageUpdate", true);
    await loadImageData();
  }

  function handleNextImage() {
    dispatch("selectNextImage");
  }

  function handlePrevImage() {
    dispatch("selectPrevImage");
  }

  function close() {
    const modal = document.getElementById("image-backdrop");
    modal.classList.add("closing");

    modal.addEventListener("animationend", () => {
      dispatch("close");
    });
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      close();
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      close();
    } else if (event.key === "ArrowRight") {
      handleNextImage();
    } else if (event.key === "ArrowLeft") {
      handlePrevImage();
    }
  }

  function handleKeydownOnEdit(event: KeyboardEvent) {
    if (event.key === "Enter") {
      renameImage();
    }
  }

  function getNewImageFileName(): string {
    if (editableFileStem.indexOf(".") !== -1) {
      editableFileStem = getFileStem(editableFileStem);
    }
    return editableFileStem.trim() + "." + getFileExtension(imageName);
  }

  async function downloadImage() {
    if (!imageDataUrl) return;

    const link = document.createElement("a");
    link.href = imageDataUrl;
    link.download = imageName;
    link.click();
  }

  function handleDeleteImage() {
    showConfirmDeleteModal = true;
  }

  function handleCancelDelete() {
    showConfirmDeleteModal = false;
  }

  function handleCloseAlertModal() {
    showAlertModal = false;
    alertText = null;
  }

  function setAlertMessage(message: string) {
    alertText = message;
    showAlertModal = true;
  }

  function resetImageName() {
    editing = false;
    editableFileStem = getFileStem(imageName);
  }

  function enableEditing() {
    editing = true;
  }

  function disableEditing(event: Event) {
    if (
      event.relatedTarget &&
      (
        event.relatedTarget.tagName === "INPUT" ||
        event.relatedTarget.id === "accept-btn"
      )
    ) {
      return;
    }

    resetImageName();
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div
  class="modal-backdrop"
  id="image-backdrop"
  onclick={handleBackdropClick}
  >
  <IconButton
    class="material-icons icon-btn"
    onclick={handlePrevImage}
    disabled={imageIds.indexOf(image.id) === 0 && !prevPageExists}
    >
    chevron_left
  </IconButton>

  <div class="modal-content">
    <IconButton
      class="material-icons icon-btn close-btn low-opac"
      onclick={close}
      aria-label="Close"
      >
      close
    </IconButton>

    <div class="image-container">
      {#if loading}
        <div class="loading-spinner">
          <div class="spinner"></div>
          <p>Loading image...</p>
        </div>
      {:else if imageDataUrl}
        <a href={imageDataUrl} target="_blank" rel="noopener noreferrer">
          <img src={imageDataUrl} alt={imageName} />
        </a>
      {:else}
        <div class="error">Failed to load image</div>
      {/if}
    </div>

    <div class="image-info">
      <div class="inner">
        <div class="image-header">
          <div class="image-name">
            {#if editing}
              <div class="name-edit">
                <input
                  type="text"
                  bind:value={editableFileStem}
                  onblur={disableEditing}
                  onkeydown={handleKeydownOnEdit}
                  autofocus
                />

                <IconButton
                  class="material-icons icon-btn"
                  id="accept-btn"
                  onclick={renameImage}
                  aria-label="Accept Edit"
                  >
                  check
                </IconButton>
              </div>
            {:else}
              <h3 onclick={enableEditing}>{imageName}</h3>
            {/if}
          </div>

          <div class="actions">
            {#if multiVersion}
              <IconButton
                class="material-icons icon-btn"
                onclick={revertImage}
                disabled={!imageDataUrl || meta.initial_version}
                >
                undo
              </IconButton>

              <IconButton
                class="material-icons icon-btn"
                onclick={restoreImage}
                disabled={!imageDataUrl || meta.latest_version}
                >
                redo
              </IconButton>
            {/if}

            <IconButton
              class="material-icons icon-btn"
              onclick={downloadImage}
              disabled={!imageDataUrl}
              >
              download
            </IconButton>

            <IconButton
              class="material-icons icon-btn delete-btn"
              onclick={handleDeleteImage}
              disabled={!imageDataUrl}
              >
              delete
            </IconButton>
          </div>
        </div>

        <div class="image-details">
          <div class="details-grid">
            <div class="detail-item">
              <span class="label">File Size</span>
              <span class="value">{formatFileSize(meta.size)}</span>
            </div>
            <div class="detail-item">
              <span class="label">Image Size</span>
              <span class="value">{meta.width} x {meta.height}</span>
            </div>
            <div class="detail-item">
              <span class="label">Type</span>
              <span class="value">{formatImageType(meta.content_type)}</span>
            </div>
            <div class="detail-item">
              <span class="label">Uploaded</span>
              <span class="value">{formatDate(meta.created_at)}</span>
            </div>
            {#if multiVersion}
              <div class="detail-item">
                <span class="label">Modified</span>
                <span class="value">{formatDate(meta.last_modified)}</span>
              </div>
              <div class="detail-item">
                <span class="label">Version</span>
                <span class="value">{meta.version_index}</span>
              </div>
            {/if}
          </div>
        </div>
      </div>
    </div>
  </div>

  <IconButton
    class="material-icons icon-btn"
    onclick={handleNextImage}
    disabled={imageIds.indexOf(image.id) === imageIds.length - 1 && !nextPageExists}
    >
    chevron_right
  </IconButton>

  {#if showConfirmDeleteModal}
    <ConfirmModal
      on:confirm={deleteImage}
      on:cancel={handleCancelDelete}
    />
  {:else if showAlertModal}
    <AlertModal
      message={alertText}
      on:close={handleCloseAlertModal}
    />
  {/if}
</div>

<style>
  .image-container {
    width: 100%;
    max-height: 500px;
    overflow: hidden;
    background: black;
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 300px;
  }

  .image-container img {
    width: 100%;
    height: auto;
    max-height: 500px;
    object-fit: contain;
  }

  .loading-spinner {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
    color: var(--im-label);
  }

  .image-info {
    background: black;
    padding: 20px;
  }

  .error {
    color: #c33;
    padding: 40px;
  }

  .image-header {
    display: flex;
    flex-wrap: wrap;
    padding: 0 24px;
    align-items: flex-start;
  }

  .image-name {
    flex-grow: 1;
  }

  .name-edit {
    margin: 14px 0 11px 0;
    display: flex;
    gap: 12px;
  }

  .name-edit input {
    all: unset;
    color: ghostwhite;
    font-style: oblique;
    font-size: 20px;
  }

  .image-header h3 {
    color: var(--im-text);
    font-size: 20px;
    word-break: break-all;
  }

  .image-header h3:hover {
    cursor: pointer;
  }

  .actions {
    margin-left: auto;
    align-self: center;
    display: flex;
    gap: 12px;
    flex-grow: 0;
  }

  .image-details {
    display: flex;
    padding: 0 24px;
  }

  .details-grid {
    display: flex;
    flex-direction: column;
    align-content: flex-start;
    row-gap: 10px;
    margin-bottom: 24px;
    width: 100%;
  }

  .detail-item {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
  }

  .label {
    flex: 1;
    font-weight: 600;
    color: var(--im-label);
    min-width: 80px;
  }

  .value {
    flex: 2;
    color: var(--im-text);
  }

  :global(.delete-btn:hover:not(:disabled)) {
    background: var(--im-warn);
  }

  :global(.delete-btn:active:not(:disabled)) {
    background: var(--im-btn-active-warn);
  }

  @media (max-width: 640px) {
    .modal-content {
      max-height: 95vh;
    }

    .image-container {
      max-height: 300px;
      min-height: 200px;
    }
  }
</style>
