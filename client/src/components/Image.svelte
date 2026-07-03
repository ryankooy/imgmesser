<script lang="ts">
  import { onMount, setContext } from "svelte";
  import { tweened } from "svelte/motion";
  import { cubicOut } from "svelte/easing";
  import Cropper from "svelte-easy-crop";
  import { Viewer } from "svelte-image-viewer";
  import { hammerSwipe } from "../utils/action.ts";
  import IconButton from "@smui/icon-button";
  import AlertModal from "./AlertModal.svelte";
  import ConfirmModal from "./ConfirmModal.svelte";
  import ImageCropper from "./ImageCropper.svelte";
  import ImageActions from "./ImageActions.svelte";
  import Transform from "./Transform.svelte";
  import {
    EditStatus, ImageStatus, ModalType, imageDataUrlCache,
  } from "../store.ts";
  import type {
    ActionModal, ActionModalButton, ConfirmModalOptions,
    ImageData, ImageMeta, Transformations
  } from "../store.ts";
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

    in(statuses: T[]): boolean {
      return statuses.includes(this.status);
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

  let actionModal: ActionModal | null = $state(null);

  let editableFileStem: string = $derived(getFileStem(imageName));

  let width: number = $derived(meta.width);
  let height: number = $derived(meta.height);

  let transformations: Transformations = $state({});
  let crop = $state({ x: 0, y: 0 });
  let zoom: number = $state(1);
  let aspect: number = $state(1);

  let panning: boolean = $state(false);

  let panViewer = $state(null);
  const containerWidth: number = 1024;
  const containerHeight: number = 512;
  const scaleX = () => containerWidth / width;
  const scaleY = () => containerHeight / height;
  const panStartScale: number = Math.min(scaleX(), scaleY());

  const scrollKeys: string[] = [
    "Space", "ArrowUp", "ArrowDown", "PageUp", "PageDown", "Home", "End",
  ];

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

    panning = status.check(ImageStatus.Panning);
    disableSwipe();
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

  async function saveImageCopy(imageCopyName: string) {
    if (imageCopyName === "") return;

    try {
      const response = await fetch(`${imageUrl(imageId)}/savecopy`, {
        method: "POST",
        headers: {"Content-Type": "application/json"},
        body: JSON.stringify({ image_name: imageCopyName }),
      });

      const data = await response.json();

      if (response.ok) {
        if (data.updated) {
          status.set(ImageStatus.Copying);
          await handleImageUpdated();
        }
      } else {
        setAlertMessage("Failed to save image copy");
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
    await updateImage(meta.version);
  }

  async function handleImageUpdated() {
    refreshImage(status);

    if (!status.in([ImageStatus.Closing, ImageStatus.Copying, ImageStatus.Deleting])) {
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
    } else if (scrollKeys.includes(event.key)) {
      event.preventDefault();
    } else if (!panning) {
      if (event.key === "ArrowRight") {
        handleNextImage();
      } else if (event.key === "ArrowLeft") {
        handlePrevImage();
      }
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
    actionModal = {
      title: "Confirm Save Current Edit",
      type: ModalType.Confirm,
      options: {
        actionText: "save this edit of",
        extraText: "All other edits will be discarded.",
        handleAction: saveImageEdits,
      },
    };

    showConfirmModal = true;
  }

  function handleSaveImageCopy() {
    actionModal = {
      title: "Save Image Copy",
      type: ModalType.SaveImageCopy,
      isConfirmType: false,
      buttons: [
        {
          text: "Save Copy",
          handleClick: saveImageCopy,
        },
      ],
    };

    showConfirmModal = true;
  }

  function handleDeleteImage() {
    actionModal = {
      title: "Confirm Delete Image",
      type: ModalType.Confirm,
      isConfirmType: true,
      options: {
        actionText: "delete",
        handleAction: deleteImage,
      },
    };

    showConfirmModal = true;
  }

  function handleDiscardCurrentEdit() {
    actionModal = {
      title: "Confirm Discard Current Edit",
      type: ModalType.Confirm,
      options: {
        actionText: "discard this edit of",
        handleAction: deleteCurrentVersion,
      },
    };

    showConfirmModal = true;
  }

  function handleDiscardAllEdits() {
    actionModal = {
      title: "Confirm Discard All Edits",
      type: ModalType.Confirm,
      options: {
        actionText: "discard all edits of",
        handleAction: async () => await discardEdits(),
      },
    };

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

  function disableSwipe() {
    const el = document.getElementById("image-backdrop") as HTMLElement;

    if (panning)
      el.classList.add("panning");
    else
      el.classList.remove("panning");
  }

  function handlePanViewerReady() {
    if (panViewer) {
      panViewer.setTransform({
        scale: panStartScale,
        x: 0,
        y: 0
      });
    }
  }

  function preventDefault(event: WheelEvent | TouchEvent) {
    event.preventDefault();
  }
</script>

<svelte:window
  on:wheel|nonpassive={preventDefault}
  on:touchmove|nonpassive={preventDefault}
  on:keydown={handleKeydown}
/>

<div
  class="modal-backdrop"
  id="image-backdrop"
  onclick={handleBackdropClick}
  use:hammerSwipe
  onswipe={handleSwipe}
  >

  {#if !panning}
    <IconButton
      class="material-icons icon-btn"
      onclick={handlePrevImage}
      disabled={imageIds.indexOf(imageId) === 0 && !prevPageExists}
      >
      chevron_left
    </IconButton>
  {/if}

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
        {:else if panning}
          <div
            class="image-viewer"
            style="width: {containerWidth}px; height: {containerHeight}px;"
            >
            <Viewer bind:this={panViewer} onready={handlePanViewerReady}>
              <img src={imageDataUrl} alt={imageName} />
            </Viewer>
          </div>
        {:else}
          <img src={imageDataUrl} alt={imageName} />
        {/if}
      {:else}
        <div class="error">Failed to load image</div>
      {/if}
    </div>

    <div class="image-info">
      {#if !panning}
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
      {/if}

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
          saveImageCopy={handleSaveImageCopy}
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

  {#if !panning}
    <IconButton
      class="material-icons icon-btn"
      onclick={handleNextImage}
      disabled={imageIds.indexOf(imageId) === imageIds.length - 1 && !nextPageExists}
      >
      chevron_right
    </IconButton>
  {/if}

  {#if showConfirmModal}
    <ConfirmModal
      props={actionModal}
      onCancel={handleModalCancel}
    />
  {:else if showAlertModal}
    <AlertModal
      message={alertText}
      on:close={handleCloseAlertModal}
    />
  {/if}
</div>

<style>
  #image-backdrop {
    background: black;
  }

  .image-modal {
    max-width: 90vw;
    max-height: fit-content;
    display: flex;
  }

  .image-container {
    width: 100%;
    height: auto;
    overflow: hidden;
    background: black;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .image-viewer {
    position: relative;
    user-select: none;
    cursor: grab;
    border: var(--im-border);
  }

  .image-viewer:active {
    cursor: grabbing;
  }

  img {
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
      overflow: visible;
    }

    .image-container {
      max-height: 100vh;
    }

    .image-details {
      font-size: 12px;
    }
  }
</style>
