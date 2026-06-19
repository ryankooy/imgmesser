<script lang="ts">
  import { onMount, setContext } from "svelte";
  import { tweened } from "svelte/motion";
  import { cubicOut } from "svelte/easing";
  import Cropper from "svelte-easy-crop";
  import { ImageViewer } from "svelte-image-viewer";
  import { hammerSwipe } from "../utils/action.ts";
  import IconButton from "@smui/icon-button";
  import AlertModal from "./AlertModal.svelte";
  import ConfirmModal from "./ConfirmModal.svelte";
  import ImageCropper from "./ImageCropper.svelte";
  import ImageActions from "./ImageActions.svelte";
  import Transform from "./Transform.svelte";
  import { EditStatus, ImageStatus, imageDataUrlCache } from "../store.ts";
  import type { ImageData, ImageMeta, Transformations } from "../store.ts";
  import { getImageDataUrl, imageUrl } from "../utils/api.ts";
  import {
    formatDate, formatFileSize, formatImageType,
    getFileExtension, getFileStem,
  } from "../utils/app.ts";

  let {
    image = null,
    imageIds = [],
    pagination = null,
    closeImage,
    refreshImage,
    selectNextImage,
    selectPrevImage,
    setImageDataUrl,
  } = $props();

  class State<T extends number> {
    private status: T = $state(0);

    set(status: T) {
      this.status = status;
    }

    get(): T {
      return this.status;
    }

    toggle(status: T) {
      this.status = (this.status !== status) ? status : 0 as T;
    }

    check(status: T): boolean {
      return this.status === status;
    }

    reset() {
      this.status = 0 as T;
    }
  }

  const meta: ImageMeta | null = $derived(image.meta ?? null);
  const imageId: string = $derived(image.id);
  let imageDataUrl: string = $derived(image.url ?? "");

  let status = $state(new State<ImageStatus>());
  let editStatus = $state(new State<EditStatus>());

  let editingName: boolean = $state(false);
  let nextPageExists: boolean = $state(false);
  let prevPageExists: boolean = $state(false);
  let showConfirmModal: boolean = $state(false);
  let showAlertModal: boolean = $state(false);
  let transformMenuOpen: boolean = $state(false);

  let alertText: string | null = $state(null);
  let modalAction: string = $state("");
  let modalActionTitle: string = $state("");
  let modalExtraText: string | null = $state(null);
  let modalConfirmFunc: () => void = $state(null);

  const imageName: string = $derived(meta.name);
  setContext("imageName", () => imageName);

  let editableFileStem: string = $derived(getFileStem(imageName));

  let width: number = $derived(meta.width);
  let height: number = $derived(meta.height);

  let transformations: Transformations = $state({});
  let crop = $state({ x: 0, y: 0 });
  let zoom: number = $state(1);
  let aspect: number = $state(1);

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
    status.set(ImageStatus.Loading);

    if ($imageDataUrlCache.has(imageId)) {
      imageDataUrl = $imageDataUrlCache.get(imageId);
    } else {
      const dataUrl = await getImageDataUrl(imageId);

      if (dataUrl) {
        setImageDataUrl(dataUrl);
        imageDataUrl = dataUrl;
      }
    }

    status.reset();
  }

  async function deleteImage() {
    try {
      const response = await fetch(`${imageUrl(imageId)}/delete`, {
        method: "POST",
      });

      if (response.ok) {
        status.set(ImageStatus.Deleting);
        await handleImageUpdated();
      } else {
        setAlertMessage("Failed to delete image");
      }
    } catch (error) {
      console.error("Error fetching:", error);
    }
  }

  async function deleteCurrentVersion() {
    try {
      const response = await fetch(`${imageUrl(imageId)}/deleteversion`, {
        method: "POST",
      });

      if (response.ok) {
        await handleImageUpdated();
      } else {
        setAlertMessage("Failed to restore image");
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
      const response = await fetch(`${imageUrl(imageId)}/rename`, {
        method: "POST",
        headers: {"Content-Type": "application/json"},
        body: JSON.stringify({ image_name: newImageName }),
      });

      const data = await response.json();

      if (response.ok) {
        if (data.updated) {
          await handleImageUpdated();
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

  async function updateImage(version: string) {
    try {
      const response = await fetch(`${imageUrl(imageId)}/update`, {
        method: "POST",
        headers: {"Content-Type": "application/json"},
        body: JSON.stringify({ version })
      });

      if (response.ok) {
        await handleImageUpdated();
      } else {
        setAlertMessage("Failed to save image");
      }
    } catch (error) {
      console.error("Error fetching:", error);
    }
  }

  function resetCrop() {
    editStatus.reset();
    zoom = aspect = 1;
    crop = { x: 0, y: 0};
  }

  function setAspect(newAspect) {
    aspect = newAspect;
  }

  async function discardEdits() {
    resetImage();
    await updateImage(meta.original_version);
  }

  async function saveImageEdits() {
    status.set(ImageStatus.Saving);
    await updateImage(meta.version);
  }

  async function handleImageUpdated() {
    refreshImage(status);

    if (!(status.check(ImageStatus.Closing) || status.check(ImageStatus.Deleting))) {
      resetImage();
      await loadImageData();
    }
  }

  function handleNextImage() {
    resetImage();
    selectNextImage();
  }

  function handlePrevImage() {
    resetImage();
    selectPrevImage();
  }

  function resetImage() {
    status.reset();
    showConfirmModal = false;

    const panButton = document.querySelector(".toggle-btn.pan-btn") as HTMLElement;
    panButton.style.color = "white";
  }

  async function handleCloseImage() {
    const modal = document.getElementById("image-backdrop");
    modal.classList.add("closing");

    modal.addEventListener("animationend", () => {
      closeImage();
    });
  }

  function toggleButtonColor(node: PointerEvent, toggled: boolean) {
    const el = node.target as HTMLElement;
    el.style.color = toggled ? "var(--im-header-gold)" : "white";
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      handleCloseImage();
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      handleCloseImage();
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

  function handleSaveImage() {
    modalAction = "save this edit of";
    modalActionTitle = "Save Current Edit";
    modalExtraText = "All other edits will be discarded"
    modalConfirmFunc = saveImageEdits;
    showConfirmModal = true;
  }

  function handleDeleteImage() {
    modalAction = "delete";
    modalActionTitle = "Delete Image";
    modalExtraText = null;
    modalConfirmFunc = deleteImage;
    showConfirmModal = true;
  }

  function handleDiscardCurrentEdit() {
    modalAction = "discard this edit of";
    modalActionTitle = "Discard Current Edit";
    modalExtraText = null;
    modalConfirmFunc = deleteCurrentVersion;
    showConfirmModal = true;
  }

  function handleDiscardAllEdits() {
    modalAction = "discard all edits of";
    modalActionTitle = "Discard All Edits";
    modalExtraText = null;
    modalConfirmFunc = discardEdits;
    showConfirmModal = true;
  }

  function handleModalCancel() {
    showConfirmModal = false;
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

  function setAnimatedRotation(degrees: number) {
    animatedRotation.set(degrees);
  }

  function clearTransformations() {
    transformations = {} as typeof Transformations;
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
    disabled={imageIds.indexOf(imageId) === 0 && !prevPageExists}
    >
    chevron_left
  </IconButton>

  <div class="modal-content image-modal">
    <div class="image-container">
      {#if status.check(ImageStatus.Loading)}
        <div class="loading-spinner">
          <div class="spinner"></div>
          <p>Loading image...</p>
        </div>
      {:else if imageDataUrl}
        {#if editStatus.check(EditStatus.Cropping)}
          <ImageCropper
            aspect={aspect}
            bind:crop={crop}
            height={height}
            imageDataUrl={imageDataUrl}
            bind:transformations={transformations}
            width={width}
            bind:zoom={zoom}
          />
        {:else if editStatus.check(EditStatus.Rotating)}
          <img
            src={imageDataUrl}
            style="transform: rotate({$animatedRotation}deg);"
            alt={imageName}
          />
        {:else if status.check(ImageStatus.Panning)}
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
          <div class="detail-item">
            <span class="label">Modified</span>
            <span class="value">{formatDate(meta.last_modified)}</span>
          </div>
        </div>
      </div>

      <div class="actions-wrapper">
        <ImageActions
          bind:editStatus
          imageDataUrl={imageDataUrl}
          imageId={imageId}
          meta={meta}
          bind:status
          bind:transformMenuOpen={transformMenuOpen}
          closeImage={handleCloseImage}
          deleteImage={handleDeleteImage}
          downloadImage={downloadImage}
          discardAllEdits={handleDiscardAllEdits}
          discardCurrentEdit={handleDiscardCurrentEdit}
          imageUpdated={handleImageUpdated}
          saveImage={handleSaveImage}
          setAlertMessage={setAlertMessage}
          toggleButtonColor={toggleButtonColor}
        />
        {#if status.check(ImageStatus.Transforming)}
          <Transform
            bind:editStatus
            height={height}
            imageDataUrl={imageDataUrl}
            imageId={imageId}
            bind:status
            bind:transformations={transformations}
            bind:transformMenuOpen={transformMenuOpen}
            width={width}
            clearTransformations={clearTransformations}
            imageUpdated={handleImageUpdated}
            resetCrop={resetCrop}
            setAlertMessage={setAlertMessage}
            setAnimatedRotation={setAnimatedRotation}
            setAspect={setAspect}
            toggleButtonColor={toggleButtonColor}
          />
        {/if}
      </div>
    </div>
  </div>

  <IconButton
    class="material-icons icon-btn"
    onclick={handleNextImage}
    disabled={imageIds.indexOf(imageId) === imageIds.length - 1 && !nextPageExists}
    >
    chevron_right
  </IconButton>

  {#if showConfirmModal}
    <ConfirmModal
      modalAction={modalAction}
      modalActionTitle={modalActionTitle}
      modalExtraText={modalExtraText}
      on:confirm={modalConfirmFunc}
      on:cancel={handleModalCancel}
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

  input {
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
  }
</style>
