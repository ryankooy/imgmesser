<script lang="ts">
  import { createEventDispatcher, onMount } from "svelte";
  import IconButton from "@smui/icon-button";
  import type { ImageData, ImageMeta } from "../store.ts";
  import { getImageDataUrl, getImageMetadata, imageGalleryUrl } from "../utils/api.ts";
  import { truncateFileName } from "../utils/app.ts";

  let {
    nextPageTrigger = 0,
    prevPageTrigger = 0,
    refreshAll = 0,
    refreshOne = 0,
    selectedId = null,
    selectingNext = false,
    selectingPrev = false
  } = $props();

  const dispatch = createEventDispatcher();

  let images: ImageMeta[] = $state([]);
  let imageDataUrls: Map<string, string> = $state(new Map());
  let imageVersions: Map<string, string> = $state(new Map());
  let loading: boolean = $state(false);
  let error: string = $state("");
  let currentPage: number = $state(1);
  let limit: number = $state(12);
  let total: number = $state(0);
  let hasMore: boolean = $state(false);
  let pageChanged: boolean = $state(false);

  let totalPages: number = $derived(Math.ceil(total / limit));

  onMount(() => {
    loadImages();
  });

  $effect(() => {
    if (refreshAll > 0) {
      currentPage = 1;
      imageDataUrls.clear();
      imageVersions.clear();
      loadImages();
      refreshAll = 0;
    }

    if (nextPageTrigger > 0) {
      nextPage();
      nextPageTrigger = 0;
      pageChanged = true;
    } else if (prevPageTrigger > 0) {
      prevPage();
      prevPageTrigger = 0;
      pageChanged = true;
    }

    if (refreshOne > 0) {
      (async () => {
        await handleUpdatedImage();
      })();
      refreshOne = 0;
    }
  });

  async function loadImages() {
    loading = true;
    error = "";

    try {
      const response = await fetch(imageGalleryUrl(currentPage, limit));
      const data = await response.json();

      if (response.ok) {
        images = data.images;
        total = data.total;
        hasMore = data.has_more;

        // Fetch actual image data for each image
        await loadImageData();

        dispatch("imagesLoaded", images);
        dispatch("totalPageCount", {
          current: currentPage,
          total: totalPages,
          more: hasMore,
        });
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
        const dataUrl = await getImageDataUrl(image.id);

        if (dataUrl) {
          imageDataUrls.set(image.id, dataUrl);
          imageVersions.set(image.id, image.version);

          // Trigger reactivity
          imageDataUrls = imageDataUrls;
          imageVersions = imageVersions;
        }
      }
    });

    await Promise.all(promises);
  }

  async function handleUpdatedImage() {
    if (selectedId) {
      let index: number = images.findIndex((img) => img.id === selectedId);

      if (selectingNext) {
        if (pageChanged) {
          selectImage(images[0]);
        } else {
          selectImage(images[++index]);
        }
      } else if (selectingPrev) {
        if (pageChanged) {
          selectImage(images[images.length - 1]);
        } else {
          selectImage(images[--index]);
        }
      } else {
        // Fetch a new data URL for the image
        const dataUrl = await getImageDataUrl(selectedId);
        if (dataUrl) {
          // Update data URLs
          imageDataUrls.set(selectedId, dataUrl);
          imageDataUrls = imageDataUrls;
        }

        // Fetch new metadata for the image
        const image = await getImageMetadata(selectedId);
        if (image) {
          // Update image versions
          imageVersions.set(selectedId, image.version);
          imageVersions = imageVersions;

          index = images.findIndex((img) => img.id === selectedId);
          if (index !== -1) {
            // Update the array of images
            images[index] = image;

            // Reselect the current image
            selectImage(image);
          }
        }
      }
    }

    pageChanged = false;
  }

  function handleUploadClick() {
    dispatch("upload");
  }

  function handleImageClick(image: ImageMeta) {
    selectImage(image);
  }

  function selectImage(image: ImageMeta) {
    const imageData: ImageData = {
      id: image.id,
      url: imageDataUrls.get(image.id),
      meta: image,
    };

    dispatch("imageSelect", imageData);
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

  function lastPage() {
    if (hasMore) {
      currentPage = totalPages;
      loadImages();
    }
  }

  function firstPage() {
    if (currentPage > 1) {
      currentPage = 1;
      loadImages();
    }
  }
</script>

<div class="gallery" id="gallery">
  <div class="inner">
    <div class="gallery-header">
      <h2>Gallery</h2>
      <span class="total-count">{total} images</span>

      <div class="upload">
        <IconButton
          class="material-icons icon-btn"
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
                    {truncateFileName(image.name)}
                  </p>
                </div>
              </div>
            </div>
          {/each}
        </div>

        <div class="pagination">
          <IconButton
            class="material-icons icon-btn"
            onclick={firstPage}
            disabled={currentPage === 1}
            >
            first_page
          </IconButton>

          <IconButton
            class="material-icons icon-btn"
            onclick={prevPage}
            disabled={currentPage === 1}
            >
            chevron_left
          </IconButton>

          <div class="page-info">
            <span class="page-numbers">
              {currentPage} / {totalPages}
            </span>
          </div>

          <IconButton
            class="material-icons icon-btn"
            onclick={nextPage}
            disabled={!hasMore}
            >
            chevron_right
          </IconButton>

          <IconButton
            class="material-icons icon-btn"
            onclick={lastPage}
            disabled={!hasMore}
            >
            last_page
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
    color: var(--im-label);
  }

  .mini-spinner {
    width: 30px;
    height: 30px;
    border: 3px solid black;
    border-top-color: var(--im-text);
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
    color: var(--im-label);
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
    text-align: center;
    transition: opacity 0.2s;
  }

  .image-card:hover .overlay {
    opacity: 1;
  }

  .image-name {
    font-weight: 600;
    color: white;
    font-size: 12px;
    width: 85%;
  }

  .pagination {
    display: flex;
    flex-direction: row;
    justify-content: center;
    align-items: center;
    gap: 14px;
    padding: 24px;
    background: var(--im-background);
    border: var(--im-border);
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
    margin: 5px;
  }

  .total-count {
    font-size: 14px;
    color: var(--im-label);
  }

  .upload {
    margin-left: auto;
    align-self: flex-start;
  }

  .gallery-header {
    display: flex;
    align-items: baseline;
    gap: 16px;
  }

  @media (max-width: 640px) {
    .grid {
      grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
      gap: 16px;
    }

    .pagination {
      gap: 8px;
    }
  }
</style>
