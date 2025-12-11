<script lang="ts">
  import { createEventDispatcher, onMount, getContext, setContext } from "svelte";
  import IconButton from "@smui/icon-button";
  import ConfirmModal from "./ConfirmModal.svelte";

  import { apiUrl } from "../store.ts";
  import type { ImageData } from "../store.ts";

  const dispatch = createEventDispatcher();

  const { image } = $props();

  let multiVersion: boolean = $derived(image.version_count > 1);
  let imageDataUrl: string = $state("");

  let loading = $state(false);
  let editing = $state(false);
  let showConfirmDeleteModal = $state(false);

  let imageName: string = $derived(image.name);
  setContext("imageName", () => imageName);

  const modalAction: string = "delete";
  setContext("modalAction", () => modalAction);

  let editableImageName = $state(getContext("imageName")());

  onMount(() => {
    loadImageData();
  });

  function imageId(): string {
    return encodeURIComponent(image.id);
  }

  async function loadImageData() {
    loading = true;
    try {
      const response = await fetch(`${apiUrl}/images/${imageId()}`);
      if (response.ok) {
        const blob = await response.blob();
        imageDataUrl = URL.createObjectURL(blob);
      }
    } catch (err) {
      console.error("Failed to load image:", err);
    } finally {
      loading = false;
    }
  }

  async function deleteImage() {
    try {
      const response = await fetch(`${apiUrl}/images/${imageId()}/delete`, {
        method: "POST",
      });

      if (response.ok) {
        dispatch("imageUpdate");
      } else {
        console.error("Failed to delete image");
      }
    } catch (err) {
      console.error("Error fetching:", err);
    }
  }

  async function renameImage() {
    editing = false;

    if (editableImageName === imageName) {
      return;
    }

    let newImageName = updateFileExtension(editableImageName);

    try {
      const response = await fetch(`${apiUrl}/images/${imageId()}/rename`, {
        method: "POST",
        headers: {"Content-Type": "application/json"},
        body: JSON.stringify({ image_name: newImageName }),
      });

      if (response.ok) {
        const data = await response.json();
        if (data.updated) {
          await handleUpdatedImage();
          editing = false;
        }
      } else {
        console.error("Failed to rename image");
      }
    } catch (err) {
      console.error("Error fetching:", err);
    }
  }

  async function revertImage() {
    try {
      const response = await fetch(`${apiUrl}/images/${imageId()}/revert`, {
        method: "POST",
      });

      if (response.ok) {
        const data = await response.json();
        if (data.updated) {
          await handleUpdatedImage();
        }
      } else {
        console.error("Failed to revert image");
      }
    } catch (err) {
      console.error("Error fetching:", err);
    }
  }

  async function restoreImage() {
    try {
      const response = await fetch(`${apiUrl}/images/${imageId()}/restore`, {
        method: "POST",
      });

      if (response.ok) {
        const data = await response.json();
        if (data.updated) {
          await handleUpdatedImage();
        }
      } else {
        console.error("Failed to restore image");
      }
    } catch (err) {
      console.error("Error fetching:", err);
    }
  }

  async function handleUpdatedImage() {
    dispatch("imageUpdate");
    await loadImageData();
  }

  function close() {
    if (imageDataUrl) {
      URL.revokeObjectURL(imageDataUrl);
    }
    dispatch("close");
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      close();
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      close();
    }
  }

  function formatFileSize(bytes: number): string {
    if (bytes < 1024) return bytes + " B";
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
    return (bytes / (1024 * 1024)).toFixed(1) + " MB";
  }

  function formatDate(dateStr: string): string {
    try {
      return new Date(dateStr).toLocaleString();
    } catch {
      return dateStr;
    }
  }

  function updateFileExtension(filename: string): string {
    let origExt = (imageName.includes(".")) ? imageName.split(".").pop() : "jpg";

    if (filename.includes(".")) {
      let ext = filename.split(".").pop();

      if (ext !== imageName) {
        const stem = filename.substring(0, filename.lastIndexOf("."));
        return stem + "." + origExt;
      } else {
        return filename;
      }
    } else {
      return filename + "." + origExt;
    }
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

    editing = false;
    editableImageName = imageName;
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="modal-backdrop" onclick={handleBackdropClick}>
  <div class="modal-content">
    <IconButton
      class="material-icons icon-btn close-btn"
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
        <img src={imageDataUrl} alt={imageName} />
      {:else}
        <div class="error">Failed to load image</div>
      {/if}
    </div>

    <div class="image-info">
      <div class="inner">
        <div class="image-header">
          {#if editing}
            <div class="name-edit">
              <input
                type="text"
                bind:value={editableImageName}
                onblur={disableEditing}
                onkeydown={(e) => {
                  if (e.key === "Enter") {
                    editing = false;
                  }
                }}
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

          <div class="actions">
            {#if multiVersion}
              <IconButton
                class="material-icons icon-btn"
                onclick={revertImage}
                disabled={!imageDataUrl || image.initial_version}
                >
                undo
              </IconButton>

              <IconButton
                class="material-icons icon-btn"
                onclick={restoreImage}
                disabled={!imageDataUrl || image.latest_version}
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
              <span class="value">{formatFileSize(image.size)}</span>
            </div>
            <div class="detail-item">
              <span class="label">Image Size</span>
              <span class="value">{image.width} x {image.height}</span>
            </div>
            <div class="detail-item">
              <span class="label">Type</span>
              <span class="value">{image.content_type}</span>
            </div>
            <div class="detail-item">
              <span class="label">Uploaded</span>
              <span class="value">{formatDate(image.created_at)}</span>
            </div>
            {#if multiVersion}
              <div class="detail-item">
                <span class="label">Modified</span>
                <span class="value">{formatDate(image.last_modified)}</span>
              </div>
              <div class="detail-item">
                <span class="label">Version</span>
                <span class="value">{image.version_index} ({image.version})</span>
              </div>
            {/if}
          </div>
        </div>
      </div>
    </div>
  </div>

  {#if showConfirmDeleteModal}
    <ConfirmModal
      on:confirm={deleteImage}
      on:cancel={handleCancelDelete}
    />
  {/if}
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
    max-width: 900px;
    width: 100%;
    max-height: 90vh;
    overflow-y: auto;
    position: relative;
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

  .spinner {
    width: 40px;
    height: 40px;
    border: 4px solid #e2e8f0;
    border-top-color: #667eea;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .image-info {
    background: black;
    padding: 20px;
  }

  .inner {
    padding: 20px;
    border: var(--im-border);
  }

  .error {
    color: #c33;
    padding: 40px;
  }

  .image-header {
    display: flex;
    padding: 0 24px;
  }

  .image-header input {
    margin: 0 0 20px 0;
    padding: 8px;
    font-size: 20px;
    border: 1px solid #667eea;
    background: #f8f9ff;
    transition: border-color 0.2s;
  }

  input[type="text"]:hover:not(:disabled) {
    border-color: #764ba2;
  }

  .image-header h3 {
    color: var(--im-text);
    font-size: 20px;
    word-break: break-all;
  }

  .image-header h3:hover {
    cursor: pointer;
  }

  .name-edit {
    display: flex;
    gap: 12px;
  }

  .actions {
    margin-left: auto;
    align-self: center;
    display: flex;
    gap: 12px;
    flex-wrap: wrap;
  }

  .image-details {
    display: flex;
    padding: 0 24px;
  }

  .details-grid {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-bottom: 24px;
  }

  .detail-item {
    display: flex;
    gap: 12px;
  }

  .label {
    font-weight: 600;
    color: var(--im-label);
    min-width: 80px;
  }

  .value {
    color: var(--im-text);
  }

  :global(.icon-btn) {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    background: none;
    color: var(--im-text);
    border: none;
    font-size: 24px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.2s;
  }

  :global(.icon-btn:hover:not(:disabled)) {
    background: var(--im-hover-gold);
  }

  :global(.icon-btn:disabled) {
    opacity: 0.5;
    cursor: not-allowed;
  }

  :global(.close-btn) {
    position: absolute;
    top: 16px;
    right: 16px;
    z-index: 10;
  }

  :global(.delete-btn) {
    color: var(--im-warn);
    transition: color 0.2s, background 0.2s;
  }

  :global(.delete-btn:hover:not(:disabled)) {
    color: var(--im-text);
    background: var(--im-warn);
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
