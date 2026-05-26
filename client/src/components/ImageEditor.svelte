<script lang="ts">
  import { createEventDispatcher, onMount, setContext } from "svelte";
  import { tweened } from "svelte/motion";
  import { cubicOut } from "svelte/easing";
  import Cropper from "svelte-easy-crop";
  import { ImageViewer, Viewer } from "svelte-image-viewer";
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

  let loading: boolean = $state(false);
  let editingName: boolean = $state(false);
  let showConfirmDeleteModal: boolean = $state(false);
  let showAlertModal: boolean = $state(false);
  let nextPageExists: boolean = $state(false);
  let prevPageExists: boolean = $state(false);

  let alertText: string | null = $state(null);

  const imageName: string = $derived(meta.name);
  setContext("imageName", () => imageName);

  let modalAction: string = "delete";
  setContext("modalAction", () => modalAction);

  let editableFileStem: string = $derived(getFileStem(imageName));

  let width: number = $derived(meta.width);
  let height: number = $derived(meta.height);

  let transformations: Transformations = $state({});
  let transforming: boolean = $state(false);

  let showConfirmDeleteEditsModal: boolean = $state(false);

  let cropping: boolean = $state(false);
  let crop = $state({ x: 0, y: 0 });
  let zoom: number = $state(1);
  let aspect: number = $state(1);

  let rotating: boolean = $state(false);
  let rotation: number = $state(0);
  let aRotation: number = $state(0);

  let resizing: boolean = $state(false);
  let resizeWidth: number = $derived(width);
  let resizeHeight: number = $derived(height);

  let panning: boolean = $state(false);

  const animatedRotation = tweened(0, {
    duration: 300,
    easing: cubicOut
  });

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

  function typeIsGif(): boolean {
    return meta.content_type === "image/gif";
  }

  function transformInProgress(): boolean {
    return Object.keys(transformations).length !== 0;
  }

  async function transformImage() {
    if (!transformInProgress()) return;
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

  function getButtonColor(toggled: boolean) {
    return toggled ? "var(--im-header-gold)" : "white";
  }

  function toggleButtonColor(node: PointerEvent, toggled: boolean) {
    const el = node.target as HTMLElement;
    el.style.color = getButtonColor(toggled);
  }

  function toggleTransform(node: PointerEvent) {
    transforming = !transforming;
    if (transforming && panning) {
      panning = false;
      const panButton = document.querySelector(".toggle-btn.pan-btn") as HTMLElement;
      panButton.style.color = getButtonColor(false);
    }
    toggleButtonColor(node, transforming);
  }

  function togglePanTool(node: PointerEvent) {
    panning = !panning;
    toggleButtonColor(node, panning);
  }

  async function rotateImageRight() {
    if (rotation === 270) {
      if (!!transformations.rotate) delete transformations.rotate;
      rotation = 0;
    } else {
      rotating = true;
      rotation += 90;
      transformations.rotate = rotation;
    }

    aRotation += 90;
    animatedRotation.set(aRotation);
  }

  async function rotateImageLeft() {
    rotating = true
    if (rotation === 0) {
      rotation = 270;
      transformations.rotate = rotation;
    } else if (rotation === 90) {
      if (!!transformations.rotate) delete transformations.rotate;
      rotation = 0;
    } else {
      rotation -= 90;
      transformations.rotate = rotation;
    }

    aRotation -= 90;
    animatedRotation.set(aRotation);
  }

  function cropImage() {
    cropping = true;
  }

  function setImgSrc(node: PointerEvent) {
    const cropperImage: HTMLImageElement = node.getElementsByClassName("svelte-easy-crop-image")[0];
    cropperImage.setAttribute("src", imageDataUrl);
  }

  function onCropComplete(details: object) {
    const cropDetails: object = details.pixels;
    if (cropDetails) {
      width = cropDetails.width;
      height = cropDetails.height;
      transformations.crop = cropDetails;
    }
  }

  function setAspect(newAspect) {
    aspect = newAspect;
  }

  async function applyEdits() {
    loading = true;

    if (resizing) resizeImage();
    await transformImage();

    resetCrop();
    resetRotation();
    resetResize();
  }

  function cancelEdits() {
    transformations = {};
    resetCrop();
    resetRotation();
    resetResize();
  }

  function resetCrop() {
    cropping = false;
    zoom = aspect = 1;
    crop = { x: 0, y: 0};
  }

  function resetRotation() {
    rotating = false;
    rotation = aRotation = 0;
    animatedRotation.set(0);
  }

  async function applyRotation() {
    loading = true;
    await transformImage();
    resetRotation();
  }

  function beginResizeImage() {
    resizing = true;
  }

  function resetResize() {
    resizing = false;
    resizeWidth = width;
    resizeHeight = height;
  }

  function resizeImage() {
    if (resizeWidth !== width || resizeHeight !== height)
      transformations.resize = { width: resizeWidth, height: resizeHeight };
  }

  function handleWidthInput(event: Event) {
    const target = event.currentTarget as HTMLInputElement;
    resizeWidth = parseInt(target.value, 10);
    resizeHeight = Math.round((resizeWidth * height) / width);
  }

  function handleHeightInput(event: Event) {
    const target = event.currentTarget as HTMLInputElement;
    resizeHeight = parseInt(target.value, 10);
    resizeWidth = Math.round((resizeHeight * width) / height);
  }

  async function saveImageEdits() {
    await updateImage("saving");
  }

  async function discardEdits() {
    showConfirmDeleteEditsModal = false;
    resetImage();
    await updateImage("canceling");
  }

  function resetImage() {
    transforming = panning = false;
    transformations = {};
    resetCrop();
    resetRotation();
    resetResize();

    const transformButton = document.querySelector(".toggle-btn.transform-btn") as HTMLElement;
    transformButton.style.color = getButtonColor(false);

    const panButton = document.querySelector(".toggle-btn.pan-btn") as HTMLElement;
    panButton.style.color = getButtonColor(false);
  }

  async function handleUpdatedImage(state?: string) {
    dispatch("imageUpdate", state);
    if (state !== "closing") {
      resetImage();
      await loadImageData();
    }
  }

  function handleNextImage() {
    resetImage();
    dispatch("selectNextImage");
  }

  function handlePrevImage() {
    resetImage();
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
      if (editingName) renameImage();
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

  function handleDiscardEdits() {
    showConfirmDeleteEditsModal = true;
  }

  function handleCancelDiscardEdits() {
    showConfirmDeleteEditsModal = false;
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

  function resetDimensions() {
    editingName = false;
    editableFileStem = getFileStem(imageName);
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
        {#if cropping}
          <div class="cropper-container" use:setImgSrc>
            <Cropper
              {imageDataUrl}
              bind:crop
              bind:zoom
              aspect={aspect}
              oncropcomplete={onCropComplete}
            />
          </div>
        {:else if rotating}
          <img
            src={imageDataUrl}
            style="transform: rotate({$animatedRotation}deg);"
            alt={imageName}
          />
        {:else if panning}
          <div class="image-viewer">
            <ImageViewer src={imageDataUrl} alt={imageName} />
          </div>
        {:else}
          <img src={imageDataUrl} alt={imageName} />
        {/if}
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
            <span class="value">{width} x {height}</span>
          </div>
          <div class="detail-item">
            <span class="label">Uploaded</span>
            <span class="value">{formatDate(meta.created_at)}</span>
          </div>
        </div>
      </div>

      <div class="actions-wrapper">
        <div class="actions-section">
          <div class="actions general">
            <!-- Transform button -->
            <IconButton
              title="Toggle transform tools"
              class="material-icons icon-btn toggle-btn transform-btn"
              onclick={toggleTransform}
              disabled={!imageDataUrl || panning || typeIsGif()}
              >
              transform
            </IconButton>
            <!-- Pan tool button -->
            <IconButton
              title="Pan tool"
              class="material-icons icon-btn toggle-btn pan-btn"
              onclick={togglePanTool}
              disabled={!imageDataUrl || transforming}
              >
              pan_tool
            </IconButton>
            <!-- Download button -->
            <IconButton
              title="Download image"
              class="material-icons icon-btn"
              onclick={downloadImage}
              disabled={!imageDataUrl}
              >
              download
            </IconButton>
            <!-- Delete button -->
            <IconButton
              title="Delete image"
              class="material-icons icon-btn delete-btn"
              onclick={handleDeleteImage}
              disabled={!imageDataUrl}
              >
              delete_forever
            </IconButton>
            <!-- Close button -->
            <IconButton
              title="Close image"
              class="material-icons icon-btn"
              onclick={close}
              aria-label="Close"
              >
              close
            </IconButton>
          </div>
        </div>

        {#if multiVersion && !typeIsGif()}
          <div class="actions-section">
            <div class="actions edits">
              <!-- Unto button -->
              <IconButton
                title="Undo change"
                class="material-icons icon-btn"
                onclick={undoEdit}
                disabled={!imageDataUrl || !multiVersion || meta.initial_version}
                >
                undo
              </IconButton>
              <!-- Redo button -->
              <IconButton
                title="Redo change"
                class="material-icons icon-btn"
                onclick={redoEdit}
                disabled={!imageDataUrl || !multiVersion || meta.latest_version}
                >
                redo
              </IconButton>
              <!-- Save button -->
              <IconButton
                title="Save current edit"
                class="material-icons icon-btn"
                onclick={saveImageEdits}
                disabled={!imageDataUrl || !multiVersion}
                >
                save
              </IconButton>
              <!-- Discard edits button -->
              <IconButton
                title="Discard all edits"
                class="material-icons icon-btn delete-btn"
                onclick={handleDiscardEdits}
                disabled={!imageDataUrl || !multiVersion}
                >
                delete_sweep
              </IconButton>
            </div>
          </div>
        {/if}

        {#if transforming}
          <div class="actions-section rotate">
            <div class="actions">
              <!-- Rotate left button -->
              <IconButton
                title="Rotate counterclockwise"
                class="material-icons icon-btn"
                onclick={rotateImageLeft}
                disabled={!imageDataUrl || cropping || resizing}
                >
                rotate_90_degrees_ccw
              </IconButton>
              <!-- Rotate right button -->
              <IconButton
                title="Rotate clockwise"
                class="material-icons icon-btn"
                onclick={rotateImageRight}
                disabled={!imageDataUrl || cropping || resizing}
                >
                rotate_90_degrees_cw
              </IconButton>
              <!-- Crop button -->
              <IconButton
                title="Crop"
                class="material-icons icon-btn"
                onclick={cropImage}
                disabled={!imageDataUrl || rotating || resizing}
                >
                crop
              </IconButton>
              <!-- Resize button -->
              <IconButton
                title="Resize"
                class="material-icons icon-btn"
                onclick={beginResizeImage}
                disabled={!imageDataUrl || cropping || rotating}
                >
                photo_size_select_small
              </IconButton>
            </div>
          </div>

          {#if cropping}
            <div class="actions-section crop">
              <div class="actions">
                <!-- Crop square button -->
                <IconButton
                  title="Crop square"
                  class="material-icons icon-btn"
                  onclick={() => setAspect(1/1)}
                  disabled={!imageDataUrl}
                  >
                  crop_square
                </IconButton>
                <!-- Crop portrait button -->
                <IconButton
                  title="Crop portrait"
                  class="material-icons icon-btn"
                  onclick={() => setAspect(4/5)}
                  disabled={!imageDataUrl}
                  >
                  crop_portrait
                </IconButton>
                <!-- Crop landscape button -->
                <IconButton
                  title="Crop landscape"
                  class="material-icons icon-btn"
                  onclick={() => setAspect(5/4)}
                  disabled={!imageDataUrl}
                  >
                  crop_landscape
                </IconButton>
                <!-- Crop 3:2 button -->
                <div
                  title="Crop 3:2"
                  class="icon-btn"
                  style="font-size: 14px;"
                  onclick={() => setAspect(3/2)}
                  disabled={!imageDataUrl}
                  >
                  3:2
                </div>
                <!-- Crop 16:9 button -->
                <div
                  title="Crop 16:9"
                  class="icon-btn"
                  style="font-size: 14px;"
                  onclick={() => setAspect(16/9)}
                  disabled={!imageDataUrl}
                  >
                  16:9
                </div>
              </div>
            </div>
          {/if}

          {#if resizing}
            <div class="actions-section resize">
              <div class="resize-input">
                <label class="form-row">
                  <span>W</span>
                  <input
                    type="number"
                    name="resize-width"
                    inputmode="numeric"
                    min="0"
                    bind:value={resizeWidth}
                    oninput={handleWidthInput}
                    autofocus
                  />
                </label>
                <label class="form-row">
                  <span>H</span>
                  <input
                    type="number"
                    name="resize-height"
                    inputmode="numeric"
                    min="0"
                    bind:value={resizeHeight}
                    oninput={handleHeightInput}
                    autofocus
                  />
                </label>
              </div>
            </div>
          {/if}

          {#if cropping || rotating || resizing}
            <div class="actions-section resize">
              <div class="actions">
                <!-- Apply edits button -->
                <IconButton
                  title="Apply resize"
                  class="material-icons icon-btn"
                  onclick={applyEdits}
                  disabled={!imageDataUrl}
                  >
                  check
                </IconButton>
                <!-- Cancel edits button -->
                <IconButton
                  title="Cancel edits"
                  class="material-icons icon-btn"
                  onclick={cancelEdits}
                  disabled={!imageDataUrl}
                  >
                  cancel
                </IconButton>
              </div>
            </div>
          {/if}
        {/if}
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
      modalAction="delete"
      modalActionTitle="Delete"
      on:confirm={deleteImage}
      on:cancel={handleCancelDelete}
    />
  {:else if showConfirmDeleteEditsModal}
    <ConfirmModal
      modalAction="discard all edits of"
      modalActionTitle="Discard Edits"
      on:confirm={discardEdits}
      on:cancel={handleCancelDiscardEdits}
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

  .image-viewer {
    position: relative;
    height: 512px;
    width: 100%;
    user-select: none;
    cursor: grab;
  }

  .image-viewer:active {
    cursor: grabbing;
  }

  .image-container img {
    width: auto;
    height: auto;
    max-width: 100%;
    max-height: 100vh;
    object-fit: contain;
    cursor: default;
    pointer-events: none;
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
    padding: 0 12px;
  }

  .error {
    color: #c33;
    padding: 40px;
  }

  .image-header {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-start;
  }

  .image-name {
    flex-grow: 1;
  }

  .name-edit {
    margin: 14px 0 11px 0;
    display: flex;
    gap: 8px;
  }

  .name-edit input {
    all: unset;
    color: ghostwhite;
    font-style: oblique;
  }

  .image-header h3 {
    color: var(--im-text);
    font-size: 16px;
    word-break: break-all;
  }

  .image-header h3:hover {
    cursor: pointer;
  }

  .actions-wrapper {
    margin-top: 10px;
  }

  .actions-section {
    padding: 2px 3px;
    margin: 3px;
  }

  .actions {
    justify-content: flex-start;
    display: flex;
    gap: 8px;
  }

  .resize-input {
    padding-left: 15px;
  }

  .form-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .form-row span {
    display: inline-block;
    width: 25px;
    color: var(--im-label);
    font-size: 14px;
  }

  .form-row input {
    flex: 1;
    all: unset;
    color: ghostwhite;
    font-style: oblique;
    font-size: 14px;
    width: 4rem;
    cursor: text;
  }

  .image-details {
    font-size: 14px;
  }

  .details-grid {
    display: flex;
    flex-wrap: wrap;
    flex-direction: column;
  }

  .detail-item {
    display: flex;
    gap: 8px;
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

  .cropper-container {
    position: relative;
    width: 100%;
    height: 75%;
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
      min-height: 50vh;
    }

    .image-details {
      font-size: 12px;
    }

    .cropper-container {
      min-height: 50vh;
    }
  }
</style>
