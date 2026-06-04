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
  import { EditState, ImageState, imageDataUrlCache } from "../store.ts";
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
    closeSelectedImage,
    handleImageUpdate,
    handleSelectDataUrl,
    handleSelectNextImage,
    handleSelectPrevImage,
  } = $props();

  const meta: ImageMeta | null = $derived(image.meta ?? null);
  const imageId: string = $derived(image.id);
  let imageDataUrl: string = $derived(image.url ?? "");

  let status: ImageState = $state(ImageState.None);
  let editStatus: EditState = $state(EditState.None);

  let editingName: boolean = $state(false);
  let showConfirmDeleteModal: boolean = $state(false);
  let showConfirmDeleteEditsModal: boolean = $state(false);
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

  function setStatus(stat: ImageState) {
    status = stat;
  }

  function getStatus(): ImageState {
    return status;
  }

  function toggleStatus(stat: ImageState) {
    status = (status !== stat) ? stat : ImageState.None;
  }

  function checkStatus(stat: ImageState): boolean {
    return status === stat;
  }

  function resetStatus() {
    status = ImageState.None;
  }

  function setEditStatus(stat: EditState) {
    editStatus = stat;
  }

  function getEditStatus(): EditState {
    return editStatus;
  }

  function toggleEditStatus(stat: EditState) {
    editStatus = (editStatus !== stat) ? stat : EditState.None;
  }

  function checkEditStatus(stat: EditState): boolean {
    return editStatus === stat;
  }

  function resetEditStatus() {
    editStatus = EditState.None;
  }

  async function loadImageData() {
    setStatus(ImageState.Loading);

    if ($imageDataUrlCache.has(imageId)) {
      imageDataUrl = $imageDataUrlCache.get(imageId);
    } else {
      const dataUrl = await getImageDataUrl(imageId);

      if (dataUrl) {
        handleSelectDataUrl(dataUrl);
        imageDataUrl = dataUrl;
      }
    }

    resetStatus();
  }

  async function deleteImage() {
    try {
      const response = await fetch(`${imageUrl(imageId)}/delete`, {
        method: "POST",
      });

      if (response.ok) {
        status = ImageState.Deleting;
        await handleUpdatedImage();
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
      const response = await fetch(`${imageUrl(imageId)}/rename`, {
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

  async function updateImage() {
    if (status === ImageState.Closing) return;

    let version: string | null = (status === ImageState.Saving) ? meta.version : meta.original_version;
    if (!version) return;

    try {
      const response = await fetch(`${imageUrl(imageId)}/update`, {
        method: "POST",
        headers: {"Content-Type": "application/json"},
        body: JSON.stringify({ version })
      });

      if (response.ok) {
        await handleUpdatedImage();
      } else {
        setAlertMessage("Failed to save image");
      }
    } catch (error) {
      console.error("Error fetching:", error);
    }
  }

  function resetCrop() {
    resetEditStatus();
    zoom = aspect = 1;
    crop = { x: 0, y: 0};
  }

  function setAspect(newAspect) {
    aspect = newAspect;
  }

  async function discardEdits() {
    showConfirmDeleteEditsModal = false;
    resetImage();
    status = ImageState.Canceling;
    await updateImage();
  }

  async function saveImageEdits() {
    status === ImageState.Saving;
    await updateImage();
  }

  async function handleUpdatedImage() {
    handleImageUpdate(status);

    if (!(status === ImageState.Closing || status === ImageState.Deleting)) {
      resetImage();
      await loadImageData();
    }
  }

  function handleNextImage() {
    resetImage();
    handleSelectNextImage();
  }

  function handlePrevImage() {
    resetImage();
    handleSelectPrevImage();
  }

  function resetImage() {
    resetStatus();

    const transformButton = document.querySelector(".toggle-btn.transform-btn") as HTMLElement;
    transformButton.style.color = "white";

    const panButton = document.querySelector(".toggle-btn.pan-btn") as HTMLElement;
    panButton.style.color = "white";
  }

  async function closeImage() {
    const modal = document.getElementById("image-backdrop");
    modal.classList.add("closing");

    modal.addEventListener("animationend", () => {
      closeSelectedImage();
    });
  }

  function toggleButtonColor(node: PointerEvent, toggled: boolean) {
    const el = node.target as HTMLElement;
    el.style.color = toggled ? "var(--im-header-gold)" : "white";
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      closeImage();
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      closeImage();
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

  function setAnimatedRotation(degrees: number) {
    animatedRotation.set(degrees);
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
      {#if status === ImageState.Loading}
        <div class="loading-spinner">
          <div class="spinner"></div>
          <p>Loading image...</p>
        </div>
      {:else if imageDataUrl}
        {#if editStatus === EditState.Cropping}
          <ImageCropper
            aspect={aspect}
            bind:crop={crop}
            height={height}
            imageDataUrl={imageDataUrl}
            bind:transformations={transformations}
            width={width}
            bind:zoom={zoom}
          />
        {:else if editStatus === EditState.Rotating}
          <img
            src={imageDataUrl}
            style="transform: rotate({$animatedRotation}deg);"
            alt={imageName}
          />
        {:else if status === ImageState.Panning}
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
        <ImageActions
          imageDataUrl={imageDataUrl}
          imageId={imageId}
          meta={meta}
          checkStatus={checkStatus}
          closeImage={closeImage}
          downloadImage={downloadImage}
          handleDeleteImage={handleDeleteImage}
          handleDiscardEdits={handleDiscardEdits}
          handleUpdatedImage={handleUpdatedImage}
          resetEditStatus={resetEditStatus}
          saveImageEdits={saveImageEdits}
          setAlertMessage={setAlertMessage}
          toggleButtonColor={toggleButtonColor}
          toggleStatus={toggleStatus}
        />
        {#if status === ImageState.Transforming}
          <Transform
            height={height}
            imageDataUrl={imageDataUrl}
            imageId={imageId}
            bind:transformations={transformations}
            width={width}
            checkEditStatus={checkEditStatus}
            getEditStatus={getEditStatus}
            handleUpdatedImage={handleUpdatedImage}
            resetCrop={resetCrop}
            resetEditStatus={resetEditStatus}
            setAlertMessage={setAlertMessage}
            setAnimatedRotation={setAnimatedRotation}
            setAspect={setAspect}
            setEditStatus={setEditStatus}
            setStatus={setStatus}
            toggleButtonColor={toggleButtonColor}
            toggleEditStatus={toggleEditStatus}
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
