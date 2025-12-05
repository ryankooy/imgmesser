<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import IconButton from "@smui/icon-button";
  import { apiUrl } from "../store.ts";
  import type { ImageData } from "../store.ts";

  const { image } = $props();

  const dispatch = createEventDispatcher();

  let imageDataUrl: string = $state("");
  let loading = $state(true);

  let multiVersion: boolean = $derived(image.version_count > 1);

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
      console.error("Image delete failed:", err);
    }
  }

  async function revertImage() {
    try {
      const response = await fetch(`${apiUrl}/images/${imageId()}/revert`, {
        method: "POST",
      });

      if (response.ok) {
        const data = await response.json();
        if (data.version_updated) {
          await handleUpdatedImage();
        }
      } else {
        console.error("Failed to revert image");
      }
    } catch (err) {
      console.error("Image revert failed:", err);
    }
  }

  async function restoreImage() {
    try {
      const response = await fetch(`${apiUrl}/images/${imageId()}/restore`, {
        method: "POST",
      });

      if (response.ok) {
        const data = await response.json();
        if (data.version_updated) {
          await handleUpdatedImage();
        }
      } else {
        console.error("Failed to restore image");
      }
    } catch (err) {
      console.error("Image restore failed:", err);
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

  async function downloadImage() {
    if (!imageDataUrl) return;

    const link = document.createElement("a");
    link.href = imageDataUrl;
    link.download = image.name;
    link.click();
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="modal-backdrop" onclick={handleBackdropClick}>
  <div class="modal-content">
    <IconButton
      class="material-icons close-btn"
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
        <img src={imageDataUrl} alt={image.name} />
      {:else}
        <div class="error">Failed to load image</div>
      {/if}
    </div>

    <div class="image-header">
      <h3>{image.name}</h3>

      <div class="actions">
        {#if multiVersion}
          <IconButton
            class="material-icons action-btn"
            onclick={revertImage}
            disabled={!imageDataUrl || image.initial_version}
            >
            undo
          </IconButton>
          <IconButton
            class="material-icons action-btn"
            onclick={restoreImage}
            disabled={!imageDataUrl || image.latest_version}
            >
            redo
          </IconButton>
        {/if}

        <IconButton
          class="material-icons action-btn"
          onclick={downloadImage}
          disabled={!imageDataUrl}
          >
          download
        </IconButton>

        <IconButton
          class="material-icons action-btn delete-btn"
          onclick={deleteImage}
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
    background: white;
    border-radius: 16px;
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
    color: #666;
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

  .error {
    color: #c33;
    padding: 40px;
  }

  .image-header {
    display: flex;
    padding: 24px 24px 0 24px;
  }

  .image-details {
    display: flex;
    padding: 0 24px;
  }

  h3 {
    margin: 0 0 20px 0;
    color: #333;
    font-size: 20px;
    word-break: break-all;
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
    color: #666;
    min-width: 80px;
  }

  .value {
    color: #333;
  }

  .actions {
    margin-left: auto;
    align-self: flex-start;
    display: flex;
    gap: 12px;
    flex-wrap: wrap;
  }

  :global(.close-btn) {
    position: absolute;
    top: 16px;
    right: 16px;
    width: 40px;
    height: 40px;
    border-radius: 50%;
    background: rgba(0, 0, 0, 0.5);
    color: white;
    border: none;
    font-size: 24px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10;
    transition: background 0.2s;
  }

  :global(.close-btn:hover) {
    background: rgba(0, 0, 0, 0.7);
  }

  :global(.action-btn) {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    background: none;
    color: black;
    border: none;
    font-size: 24px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.2s;
  }

  :global(.action-btn:hover:not(:disabled)) {
    background: #e0e0e0;
  }

  :global(.action-btn:disabled) {
    opacity: 0.5;
    cursor: not-allowed;
  }

  :global(.delete-btn) {
    color: darkred;
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
