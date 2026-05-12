<script lang="ts">
  import { createEventDispatcher, onMount, setContext } from "svelte";
  import { tweened } from "svelte/motion";
  import { cubicOut } from "svelte/easing";
  import { hammerSwipe } from "../utils/action.ts";
  import IconButton from "@smui/icon-button";
  import AlertModal from "./AlertModal.svelte";
  import ConfirmModal from "./ConfirmModal.svelte";
  import { imageDataUrlCache } from "../store.ts";
  import type { ImageMeta, ImageData, Transformations } from "../store.ts";
  import { getImageDataUrl, imageUrl } from "../utils/api.ts";
  import {
    formatDate, formatFileSize, formatImageType,
    getFileExtension, getFileStem,
  } from "../utils/app.ts";

  const dispatch = createEventDispatcher();

  let {
    image = null,
    imageIds = [],
    pagination = null,
  } = $props();

  const meta: ImageMeta | null = $derived(image.meta ?? null);
  let imageDataUrl: string = $derived(image.url ?? "");

  const multiVersion: boolean = $derived(meta.version_count > 1);

  let transformations: Transformations = $state({});
  let loading: boolean = $state(false);
  let editingName: boolean = $state(false);
  let showConfirmDeleteModal: boolean = $state(false);
  let showAlertModal: boolean = $state(false);
  let nextPageExists: boolean = $state(false);
  let prevPageExists: boolean = $state(false);
  let rotation: number = $state(0);
  let rotation_counter: number = $state(0);

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
    if (pagination) {
      nextPageExists = pagination.has_more;
      prevPageExists = pagination.current_page > 1;
    }
  });

  async function loadImageData() {
    loading = true;

    if ($imageDataUrlCache.has(image.id)) {
      imageDataUrl = $imageDataUrlCache.get(image.id);
    } else {
      const dataUrl = await getImageDataUrl(image.id);

      if (dataUrl) {
        dispatch("selectDataUrl", dataUrl);
        imageDataUrl = dataUrl;
      }
    }

    loading = false;
  }

  async function deleteImage() {
    try {
      const response = await fetch(`${imageUrl(image.id)}/delete`, {
        method: "POST",
      });

      if (response.ok) {
        dispatch("imageUpdate", "deleting");
      } else {
        setAlertMessage("Failed to delete image");
      }
    } catch (error) {
      console.error("Error fetching:", error);
    }
  }

  async function renameImage() {
    editingName = false;

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
    } catch (error) {
      console.error("Error fetching:", error);
    }
  }

  async function undoEdit() {
    try {
      const response = await fetch(`${imageUrl(image.id)}/revert`, {
        method: "POST",
      });

      if (response.ok) {
        const data = await response.json();
        if (data.updated) {
          $imageDataUrlCache.delete(image.id);
          await handleUpdatedImage();
        }
      } else {
        setAlertMessage("Failed to revert image");
      }
    } catch (error) {
      console.error("Error fetching:", error);
    }
  }

  async function redoEdit() {
    try {
      const response = await fetch(`${imageUrl(image.id)}/restore`, {
        method: "POST",
      });

      if (response.ok) {
        const data = await response.json();
        if (data.updated) {
          $imageDataUrlCache.delete(image.id);
          await handleUpdatedImage();
        }
      } else {
        setAlertMessage("Failed to restore image");
      }
    } catch (error) {
      console.error("Error fetching:", error);
    }
  }

  async function updateImage(state: string) {
    if (state === "closing") return;
    let version: string | null = (state === "saving") ? meta.version : meta.original_version;
    if (!version) return;
    try {
      const response = await fetch(`${imageUrl(image.id)}/update`, {
        method: "POST",
        headers: {"Content-Type": "application/json"},
        body: JSON.stringify({ version })
      });

      if (response.ok) {
        await handleUpdatedImage(state);
      } else {
        setAlertMessage("Failed to save image");
      }
    } catch (error) {
      console.error("Error fetching:", error);
    }
  }

  async function getBytesFromDataUrl(dataUrl: string): Promise<Uint8Array> {
    const response = await fetch(dataUrl);
    const buffer = await response.arrayBuffer();
    return new Uint8Array(buffer);
  }

  function transformInProgress(): boolean {
    return Object.keys(transformations).length !== 0;
  }

  async function transformImage() {
    if (!transformInProgress()) return;
    loading = true;
    try {
      const response = await fetch(`${imageUrl(image.id)}/transform`, {
        method: "POST",
        headers: {"Content-Type": "application/json"},
        body: JSON.stringify(transformations),
      });

      if (response.ok) {
        await handleUpdatedImage("editing");
      } else {
        setAlertMessage("Failed to transform image");
      }
    } catch (error) {
      console.error("Error fetching:", error);
    }
  }

  async function rotateImageRight() {
    transformations.rotate = 90;
    await transformImage();
  }

  async function rotateImageLeft() {
    transformations.rotate = 270;
    await transformImage();
  }

  async function saveImageEdits() {
    await updateImage("saving");
  }

  async function cancelEdits() {
    await updateImage("canceling");
  }

  async function handleUpdatedImage(state?: string) {
    dispatch("imageUpdate", state);
    if (state !== "closing")
      await loadImageData();
  }

  function handleNextImage() {
    dispatch("selectNextImage");
  }

  function handlePrevImage() {
    dispatch("selectPrevImage");
  }

  async function close() {
    await updateImage("closing");
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

  function handleSwipe(event: CustomEvent<{ direction: string }>) {
    if (event.detail.direction === "swiperight") {
      handlePrevImage();
    } else if (event.detail.direction === "swipeleft") {
      handleNextImage();
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
    editingName = false;
    editableFileStem = getFileStem(imageName);
  }

  function enableNameEditing() {
    editingName = true;
  }

  function disableNameEditing(event: Event) {
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
  use:hammerSwipe
  onswipe={handleSwipe}
  >
  <IconButton
    class="material-icons icon-btn"
    onclick={handlePrevImage}
    disabled={imageIds.indexOf(image.id) === 0 && !prevPageExists}
    >
    chevron_left
  </IconButton>

  <div class="modal-content image-modal">
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
      <div class="image-header">
        <div class="image-name">
          {#if editingName}
            <div class="name-edit">
              <input
                type="text"
                bind:value={editableFileStem}
                onblur={disableNameEditing}
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
            <h3 onclick={enableNameEditing}>{editableFileStem}</h3>
          {/if}
        </div>
      </div>

      <div class="image-details">
        <div class="details-grid">
          <div class="detail-item">
            <span class="label">Type</span>
            <span class="value">{formatImageType(meta.content_type)}</span>
          </div>
          <div class="detail-item">
            <span class="label">File Size</span>
            <span class="value">{formatFileSize(meta.size)}</span>
          </div>
          <div class="detail-item">
            <span class="label">Image Size</span>
            <span class="value">{meta.width} x {meta.height}</span>
          </div>
          <div class="detail-item">
            <span class="label">Uploaded</span>
            <span class="value">{formatDate(meta.created_at)}</span>
          </div>
        </div>
      </div>

      <div class="actions">
        <IconButton
          title="Download image"
          class="material-icons icon-btn"
          onclick={downloadImage}
          disabled={!imageDataUrl}
          >
          download
        </IconButton>
        <IconButton
          title="Delete image"
          class="material-icons icon-btn delete-btn"
          onclick={handleDeleteImage}
          disabled={!imageDataUrl}
          >
          delete
        </IconButton>
        <IconButton
          title="Close image"
          class="material-icons icon-btn"
          onclick={close}
          aria-label="Close"
          >
          close
        </IconButton>
      </div>

      <div class="actions">
        <IconButton
          title="Undo change"
          class="material-icons icon-btn"
          onclick={undoEdit}
          disabled={!imageDataUrl || !multiVersion || meta.initial_version}
          >
          undo
        </IconButton>
        <IconButton
          title="Redo change"
          class="material-icons icon-btn"
          onclick={redoEdit}
          disabled={!imageDataUrl || !multiVersion || meta.latest_version}
          >
          redo
        </IconButton>
        <IconButton
          title="Cancel editing"
          class="material-icons icon-btn"
          onclick={cancelEdits}
          disabled={!imageDataUrl || !multiVersion}
          >
          cancel
        </IconButton>
        <IconButton
          title="Save edit"
          class="material-icons icon-btn"
          onclick={saveImageEdits}
          disabled={!imageDataUrl || !multiVersion}
          >
          save
        </IconButton>
      </div>

      <div class="actions">
        <IconButton
          title="Rotate counterclockwise"
          class="material-icons icon-btn"
          onclick={rotateImageLeft}
          disabled={!imageDataUrl}
          >
          rotate_90_degrees_ccw
        </IconButton>
        <IconButton
          title="Rotate clockwise"
          class="material-icons icon-btn"
          onclick={rotateImageRight}
          disabled={!imageDataUrl}
          >
          rotate_90_degrees_cw
        </IconButton>
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
  .image-modal {
    max-width: 90vw;
    height: 100vh;
    max-height: fit-content;
    display: flex;
  }

  .image-container {
    width: 100%;
    overflow: hidden;
    background: black;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .image-container img {
    width: auto;
    height: auto;
    max-width: 100%;
    max-height: 100vh;
    object-fit: contain;
    cursor: default;
  }

  .loading-spinner {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
    color: var(--im-label);
  }

  .image-info {
    display: flex;
    flex-direction: column;
    justify-content: center;
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
    font-size: 16px;
    word-break: break-all;
  }

  .image-header h3:hover {
    cursor: pointer;
  }

  .actions {
    align-self: center;
    display: flex;
    gap: 12px;
    flex-grow: 0;
  }

  .image-details {
    padding: 0 24px 12px 24px;
    font-size: 14px;
  }

  .details-grid {
    display: flex;
    flex-wrap: wrap;
    flex-direction: column;
  }

  .detail-item {
    display: flex;
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
    .image-modal {
      display: block;
    }

    .image-container {
      max-height: 100vh;
    }

    .image-details {
      font-size: 12px;
    }
  }
</style>
