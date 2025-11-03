<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import { API_URL } from "../store.ts";
  import type { ImageData } from "../App.svelte";

  export let refresh = 0;
  const dispatch = createEventDispatcher();

  let images: ImageData[] = [];
  let imageDataUrls: Map<string, string> = new Map();
  let loading = false;
  let error = "";
  let currentPage = 1;
  let limit = 12;
  let total = 0;
  let hasMore = false;

  $: totalPages = Math.ceil(total / limit);

  onMount(() => {
    loadImages();
  });

  $: if (refresh > 0) {
    currentPage = 1;
    imageDataUrls.clear();
    loadImages();
  }

  async function loadImages() {
    loading = true;
    error = "";

    try {
      const response = await fetch(
        `${API_URL}/images?page=${currentPage}&limit=${limit}`
      );
      const data = await response.json();

      if (response.ok && data.success) {
        images = data.images;
        total = data.total;
        hasMore = data.has_more;

        // Fetch actual image data for each image
        await loadImageData();
      } else {
        error = "Failed to load images";
      }
    } catch (err) {
      error = `Error: ${err}`;
      console.error("Load error:", err);
    } finally {
      loading = false;
    }
  }

  async function loadImageData() {
    // Load images in parallel
    const promises = images.map(async (image) => {
      if (!imageDataUrls.has(image.key)) {
        try {
          const response = await fetch(`${API_URL}/images/${encodeURIComponent(image.key)}`);
          if (response.ok) {
            const blob = await response.blob();
            const dataUrl = URL.createObjectURL(blob);
            imageDataUrls.set(image.key, dataUrl);
            imageDataUrls = imageDataUrls; // Trigger reactivity
          }
        } catch (err) {
          console.error(`Failed to load image ${image.key}:`, err);
        }
      }
    });

    await Promise.all(promises);
  }

  function handleImageClick(image: ImageData) {
    dispatch("imageSelect", image);
  }

  function goToPage(page: number) {
    currentPage = page;
    loadImages();
  }

  function nextPage() {
    if (hasMore) {
      currentPage++;
      loadImages();
    }
  }

  function prevPage() {
    if (currentPage > 1) {
      currentPage--;
      loadImages();
    }
  }

  function formatFileSize(bytes: number): string {
    if (bytes < 1024) return bytes + " B";
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
    return (bytes / (1024 * 1024)).toFixed(1) + " MB";
  }

  function formatDate(dateStr: string): string {
    try {
      return new Date(dateStr).toLocaleDateString();
    } catch {
      return dateStr;
    }
  }
</script>

<div class="gallery" id="gallery">
  <h2>Gallery</h2>

  <div class="gallery-section">
    {#if loading}
      <div class="loading">
        <div class="spinner"></div>
        <p>Loading images...</p>
      </div>
    {:else if error}
      <div class="error-message">
        {error}
        <button on:click={loadImages}>Retry</button>
      </div>
    {:else if images.length === 0}
      <div class="empty-state">
        <p>No images here.</p>
      </div>
    {:else}
      <div class="grid">
        {#each images as image (image.key)}
          <div
            class="image-card"
            role="button"
            tabindex="0"
            aria-label="Image viewer"
            on:click={() => handleImageClick(image)}
            >
            <div class="image-wrapper">
              {#if imageDataUrls.has(image.key)}
                <img src={imageDataUrls.get(image.key)} alt={image.key} />
              {:else}
                <div class="image-loading">
                  <div class="mini-spinner"></div>
                </div>
              {/if}
              <div class="overlay" />
            </div>
            <div class="image-info">
              <p class="image-name" title={image.key}>
                {image.key.length > 25 ? image.key.substring(0, 25) + "..." : image.key}
              </p>
              <div class="meta">
                <span>{formatFileSize(image.size)}</span>
                <span>•</span>
                <span>{formatDate(image.last_modified)}</span>
              </div>
            </div>
          </div>
        {/each}
      </div>

      <div class="pagination">
        <button
          on:click={prevPage}
          disabled={currentPage === 1}
          class="page-btn"
        >
          ← Previous
        </button>

        <div class="page-info">
          <span class="page-numbers">
            Page {currentPage} of {totalPages}
          </span>
          <span class="total-count">({total} total images)</span>
        </div>

        <button
          on:click={nextPage}
          disabled={!hasMore}
          class="page-btn"
        >
          Next →
        </button>
      </div>
    {/if}
  </div>
</div>

<style>
  .gallery {
    min-height: 400px;
    background: white;
    border-radius: 16px;
    padding: 32px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }

  h2 {
    margin: 0 0 24px 0;
    color: #333;
    font-size: 24px;
  }

  .gallery-section {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 60px 20px;
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

  .mini-spinner {
    width: 30px;
    height: 30px;
    border: 3px solid #e2e8f0;
    border-top-color: #667eea;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .error-message {
    background: #fee;
    border: 1px solid #fcc;
    color: #c33;
    padding: 20px;
    border-radius: 8px;
    text-align: center;
  }

  .error-message button {
    margin-top: 12px;
    padding: 8px 16px;
    background: #c33;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  .empty-state {
    text-align: center;
    padding: 60px 20px;
    color: #666;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
    gap: 24px;
    margin-bottom: 32px;
  }

  .image-card {
    background: white;
    border-radius: 12px;
    overflow: hidden;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    cursor: pointer;
    transition: transform 0.2s, box-shadow 0.2s;
  }

  .image-card:hover {
    transform: translateY(-4px);
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.15);
  }

  .image-wrapper {
    position: relative;
    width: 100%;
    padding-top: 75%;
    overflow: hidden;
    background: #f0f0f0;
  }

  .image-wrapper img {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .image-loading {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #f8f9fa;
  }

  .overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    opacity: 0;
    transition: opacity 0.2s;
  }

  .image-card:hover .overlay {
    opacity: 1;
  }

  .image-info {
    padding: 12px;
  }

  .image-name {
    margin: 0 0 8px 0;
    font-weight: 600;
    color: #333;
    font-size: 14px;
  }

  .meta {
    display: flex;
    gap: 8px;
    font-size: 12px;
    color: #666;
  }

  .pagination {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 16px;
    padding: 24px;
    background: white;
    border-radius: 12px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }

  .page-btn {
    padding: 10px 20px;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    border: none;
    border-radius: 8px;
    font-weight: 600;
    cursor: pointer;
    transition: transform 0.2s, opacity 0.2s;
  }

  .page-btn:hover:not(:disabled) {
    transform: scale(1.05);
  }

  .page-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
    transform: none;
  }

  .page-info {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
  }

  .page-numbers {
    font-weight: 600;
    color: #333;
  }

  .total-count {
    font-size: 14px;
    color: #666;
  }

  @media (max-width: 640px) {
    .grid {
      grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
      gap: 16px;
    }

    .pagination {
      flex-direction: column;
      gap: 12px;
    }

    .page-btn {
      width: 100%;
    }
  }
</style>
