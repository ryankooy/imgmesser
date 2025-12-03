<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import { apiUrl } from "../store.ts";
  import type { ImageData } from "../store.ts";

  export let image: ImageData;

  const dispatch = createEventDispatcher();

  let imageDataUrl: string = "";
  let loading = true;
  let multiVersion: boolean = image.version_count > 1;

  const undoSymbol = "↶";
  const redoSymbol = "↷";

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

  async function revertImage() {
    try {
      const response = await fetch(`${apiUrl}/images/${imageId()}/revert`, {
        method: "POST",
      });

      if (response.ok) {
        const data = await response.json();

        if (data.version_updated) {
          dispatch("imageUpdate");
        }
      } else {
        console.error("Failed to revert image");
      }
    } catch (err) {
      console.error("Image reversion failed:", err);
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
          dispatch("imageUpdate");
        }
      } else {
        console.error("Failed to restore image");
      }
    } catch (err) {
      console.error("Image restoration failed:", err);
    }
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

<div class="modal-backdrop" on:click={handleBackdropClick}>
  <div class="modal-content">
    <button class="close-btn" on:click={close} aria-label="Close">
      ✕
    </button>

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

    <div class="image-details">
      <h3>{image.name}</h3>
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

      {#if multiVersion}
        <div class="actions">
          <button
            class="action-btn"
            on:click={revertImage}
            disabled={!imageDataUrl || image.initial_version}
            >
            {undoSymbol}
          </button>
          <button
            class="action-btn"
            on:click={restoreImage}
            disabled={!imageDataUrl || image.latest_version}
            >
            {redoSymbol}
          </button>
        </div>
        <br />
      {/if}
      <div class="actions">
        <button class="action-btn download" on:click={downloadImage} disabled={!imageDataUrl}>
          Download
        </button>
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

  .close-btn {
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

  .close-btn:hover {
    background: rgba(0, 0, 0, 0.7);
  }

  .image-container {
    width: 100%;
    max-height: 500px;
    overflow: hidden;
    background: #f0f0f0;
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

  .image-details {
    padding: 24px;
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
    gap: 12px;
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
    display: flex;
    gap: 12px;
    flex-wrap: wrap;
  }

  .action-btn {
    flex: 1;
    min-width: 140px;
    padding: 12px 20px;
    background: #f0f0f0;
    border: 1px solid #ddd;
    border-radius: 8px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .action-btn:hover:not(:disabled) {
    background: #e0e0e0;
    transform: translateY(-2px);
  }

  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .action-btn.download {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    border: none;
  }

  .action-btn.download:hover:not(:disabled) {
    box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
  }

  @media (max-width: 640px) {
    .modal-content {
      max-height: 95vh;
    }

    .image-container {
      max-height: 300px;
      min-height: 200px;
    }

    .actions {
      flex-direction: column;
    }

    .action-btn {
      width: 100%;
    }
  }
</style>
