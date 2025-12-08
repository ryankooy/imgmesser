<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import IconButton from "@smui/icon-button";
  import { apiUrl } from "../store.ts";
  import type { ImageData } from "../store.ts";

  let { refresh = 0 } = $props();
  const dispatch = createEventDispatcher();

  let images: ImageData[] = $state([]);
  let imageDataUrls: Map<string, string> = $state(new Map());
  let imageVersions: Map<string, string> = $state(new Map());
  let loading: boolean = $state(false);
  let error: string = $state("");
  let currentPage: number = $state(1);
  let limit: number = $state(12);
  let total: number = $state(0);
  let hasMore: boolean = $state(false);

  let totalPages: number = $derived(Math.ceil(total / limit));

  onMount(() => {
    loadImages();
  });

  $effect(() => {
    if (refresh > 0) {
      currentPage = 1;
      imageDataUrls.clear();
      imageVersions.clear();
      loadImages();
      refresh = 0;
    }
  });

  async function loadImages() {
    loading = true;
    error = "";

    try {
      const response = await fetch(
        `${apiUrl}/images?page=${currentPage}&limit=${limit}`
      );
      const data = await response.json();

      if (response.ok) {
        images = data.images;
        total = data.total;
        hasMore = data.has_more;

        dispatch("imagesLoaded", images);

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
      let versionChanged = false;

      if (imageVersions.has(image.id)) {
        const currentVersion = imageVersions.get(image.id);
        versionChanged = image.version !== currentVersion;
      }

      if (!imageDataUrls.has(image.id) || versionChanged) {
        try {
          const response = await fetch(`${apiUrl}/images/${encodeURIComponent(image.id)}`);
          if (response.ok) {
            const blob = await response.blob();
            const dataUrl = URL.createObjectURL(blob);
            imageDataUrls.set(image.id, dataUrl);
            imageVersions.set(image.id, image.version);

            // Trigger reactivity
            imageDataUrls = imageDataUrls;
            imageVersions = imageVersions;
          }
        } catch (err) {
          console.error(`Failed to load image ${image.name}:`, err);
        }
      }
    });

    await Promise.all(promises);
  }

  function handleUploadClick() {
    dispatch("upload");
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
</script>

<div class="gallery" id="gallery">
  <div class="inner">
    <div class="gallery-header">
      <h2>Gallery</h2>

      <div class="upload">
        <IconButton
          class="material-icons btn"
          onclick={handleUploadClick}
          >
          add
        </IconButton>
      </div>
    </div>

    <div class="gallery-section">
      {#if loading}
        <div class="loading">
          <div class="spinner"></div>
          <p>Loading images...</p>
        </div>
      {:else if error}
        <div class="error-message">
          {error}
          <button onclick={loadImages}>Retry</button>
        </div>
      {:else if images.length === 0}
        <div class="empty-state">
          <p>No images here.</p>
        </div>
      {:else}
        <div class="grid">
          {#each images as image (image.name)}
            <div
              class="image-card"
              role="button"
              tabindex="0"
              aria-label="Image viewer"
              onclick={() => handleImageClick(image)}
              >
              <div class="image-wrapper">
                {#if imageDataUrls.has(image.id)}
                  <img src={imageDataUrls.get(image.id)} alt={image.name} />
                {:else}
                  <div class="image-loading">
                    <div class="mini-spinner"></div>
                  </div>
                {/if}
                <div class="overlay">
                  <p class="image-name" title={image.name}>
                    {image.name.length > 25 ? image.name.substring(0, 25) + "..." : image.name}
                  </p>
                </div>
              </div>
            </div>
          {/each}
        </div>

        <div class="pagination">
          <IconButton
            class="material-icons btn"
            onclick={prevPage}
            disabled={currentPage === 1}
            >
            chevron_left
          </IconButton>

          <div class="page-info">
            <span class="page-numbers">
              Page {currentPage} of {totalPages}
            </span>
            <span class="total-count">({total} total images)</span>
          </div>

          <IconButton
            class="material-icons btn"
            onclick={nextPage}
            disabled={!hasMore}
            >
            chevron_right
          </IconButton>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .gallery {
    min-height: 400px;
    background: none;
    padding: 20px;
  }

  h2 {
    margin: 0 0 24px 0;
    color: var(--im-header-gold);
    font-size: 24px;
  }

  .gallery-section {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .inner {
    padding: 20px;
    border: var(--im-border);
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
    text-align: center;
  }

  .error-message button {
    margin-top: 12px;
    padding: 8px 16px;
    background: #c33;
    color: white;
    border: none;
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
    overflow: hidden;
    cursor: pointer;
    transition: transform 0.2s;
  }

  .image-card:hover {
    transform: translateY(-4px);
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
    display: flex;
    justify-content: center;
    align-items: center;
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

  .image-name {
    font-weight: 600;
    color: white;
    font-size: 14px;
    display: none;
  }

  .image-card:hover .image-name {
    display: block;
  }

  .pagination {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 16px;
    padding: 24px;
    background: var(--img-background-purple);
    border: var(--im-border);
  }

  :global(.btn) {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    background: none;
    color: ghostwhite;
    border: none;
    font-size: 24px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.2s;
  }

  :global(.btn:hover:not(:disabled)) {
    background: rgba(255, 255, 255, 0.1);
  }

  :global(.btn:disabled) {
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

  .upload {
    margin-left: auto;
    align-self: flex-start;
  }

  .gallery-header {
    display: flex;
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

    :global(.btn) {
      width: 100%;
    }
  }
</style>
