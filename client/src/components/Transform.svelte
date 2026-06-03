<script lang="ts">
  import IconButton from "@smui/icon-button";
  import { imageUrl } from "../utils/api.ts";
  import { EditState, ImageState } from "../store.ts";

  let {
    height = 0,
    imageDataUrl = "",
    imageId = null,
    transformations = {},
    width = 0,
    checkEditStatus,
    handleUpdatedImage,
    resetCrop,
    resetEditStatus,
    setAlertMessage,
    setAnimatedRotation,
    setAspect,
    setEditStatus,
    setStatus,
    toggleButtonColor,
    toggleEditStatus,
  } = $props();

  let rotation: number = $state(0);
  let aRotation: number = $state(0);
  let rotateApplied: boolean = $state(false);

  let resizeWidth: number = $derived(width);
  let resizeHeight: number = $derived(height);

  let settingFilters: boolean = $state(false);
  let grayscaling: boolean = $state(false);
  let grayscaleApplied: boolean = $state(false);

  let grayscaleMorph: string | null = $state(null);
  let grayscaleMask: string = $state("square");
  let grayscaleRadius: number = $state(1);

  let sepia: boolean = $state(false);

  async function transformImage() {
    if (Object.keys(transformations).length === 0) return;
    try {
      const response = await fetch(`${imageUrl(imageId)}/transform`, {
        method: "POST",
        headers: {"Content-Type": "application/json"},
        body: JSON.stringify(transformations),
      });

      if (response.ok) {
        await handleUpdatedImage("editing");
        resetEdits();
      } else {
        setAlertMessage("Failed to transform image");
      }
    } catch (error) {
      console.error("Error fetching:", error);
    }
  }

  function toggleRotate(node: PointerEvent) {
    toggleEditStatus(EditState.Rotating);
    toggleButtonColor(node, checkEditStatus(EditState.Rotating));

    if (!checkEditStatus(EditState.Rotating)) resetRotate();
  }

  function toggleFilters(node: PointerEvent) {
    toggleEditStatus(EditState.SettingFilters);
    toggleButtonColor(node, checkEditStatus(EditState.SettingFilters));

    if (!checkEditStatus(EditState.SettingFilters)) resetFilters();
  }

  async function rotateImageRight() {
    if (rotation === 270) {
      if (!!transformations.rotate) delete transformations.rotate;
      rotation = 0;
    } else {
      setEditStatus(EditState.Rotating);
      rotation += 90;
      transformations.rotate = rotation;
      rotateApplied = true;
    }

    aRotation += 90;
    setAnimatedRotation(aRotation);
  }

  async function rotateImageLeft() {
    setEditStatus(EditState.Rotating);

    if (rotation === 0) {
      rotation = 270;
      transformations.rotate = rotation;
      rotateApplied = true;
    } else if (rotation === 90) {
      if (!!transformations.rotate) delete transformations.rotate;
      rotation = 0;
    } else {
      rotation -= 90;
      transformations.rotate = rotation;
      rotateApplied = true;
    }

    aRotation -= 90;
    setAnimatedRotation(aRotation);
  }

  async function applyEdits() {
    setStatus(ImageState.Loading);

    if (checkEditStatus(EditState.Resizing)) resizeImage();
    if (grayscaling) grayscaleImage();

    await transformImage();
  }

  function resetEdits() {
    transformations = {};

    // Reset rotate button color
    const rotateBtn = document.querySelector(".toggle-btn.rotate-btn") as HTMLElement;
    if (rotateBtn) rotateBtn.style.color = "white";

    // Reset filters button color
    const filtersBtn = document.querySelector(".toggle-btn.filters-btn") as HTMLElement;
    if (filtersBtn) filtersBtn.style.color = "white";

    resetCrop();
    resetRotate();
    resetResize();
    resetFilters();
    resetEditStatus();
  }

  function resetRotate() {
    rotateApplied = false;
    rotation = aRotation = 0;
    setAnimatedRotation(0);
  }

  function resizeImage() {
    if (resizeWidth !== width || resizeHeight !== height)
      transformations.resize = { width: resizeWidth, height: resizeHeight };
  }

  function resetResize() {
    resizeWidth = width;
    resizeHeight = height;
  }

  function toggleGrayscale() {
    grayscaling = !grayscaling;
    transformations.filters ??= {};
    transformations.filters.grayscale = grayscaling;

    if (!grayscaling) resetGrayscale();
  }

  function grayscaleImage() {
    if (!grayscaleMorph) return;

    let grayscaleOptions: object = {
      morphology: grayscaleMorph,
      mask: grayscaleMask,
      radius: grayscaleRadius,
    };

    transformations.filters.options ??= {};
    Object.assign(transformations.filters.options, grayscaleOptions);
  }

  function resetGrayscale() {
    grayscaling = grayscaleApplied = false;
    grayscaleMorph = null;
    grayscaleMask = "square";
    grayscaleRadius = 1;
  }

  function setGrayscaleMorph(morphology: string) {
    grayscaleMorph = morphology;
    grayscaleApplied = true;
  }

  function setGrayscaleMask(mask: string) {
    grayscaleMask = mask;
  }

  function toggleSepia() {
    sepia = !sepia;
    transformations.filters ??= {};
    transformations.filters.sepia = sepia;
  }

  function resetFilters() {
    resetEditStatus();
    resetGrayscale();
    sepia = false;
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

  function otherEditInProgress(stat: EditState): boolean {
    return !(checkEditStatus(stat) || checkEditStatus(EditState.None));
  }
</script>

<div>
  <div class="actions-section">
    <div class="actions">
      <!-- Rotate button -->
      <IconButton
        title="Rotate"
        class="material-icons icon-btn toggle-btn rotate-btn"
        onclick={toggleRotate}
        disabled={!imageDataUrl || otherEditInProgress(EditState.Rotating)}
        >
        rotate_left
      </IconButton>
      <!-- Crop button -->
      <IconButton
        title="Crop"
        class="material-icons icon-btn"
        onclick={() => setEditStatus(EditState.Cropping)}
        disabled={!imageDataUrl || otherEditInProgress(EditState.Cropping)}
        >
        crop
      </IconButton>
      <!-- Resize button -->
      <IconButton
        title="Resize"
        class="material-icons icon-btn"
        onclick={() => setEditStatus(EditState.Resizing)}
        disabled={!imageDataUrl || otherEditInProgress(EditState.Resizing)}
        >
        photo_size_select_small
      </IconButton>
      <!-- Filters button -->
      <IconButton
        title="Filters"
        class="material-icons icon-btn toggle-btn filters-btn"
        onclick={toggleFilters}
        disabled={!imageDataUrl || otherEditInProgress(EditState.SettingFilters)}
        >
        filter_b_and_w
      </IconButton>
    </div>
  </div>

  {#if checkEditStatus(EditState.Rotating)}
    <div class="actions-section">
      <div class="actions">
        <!-- Rotate left button -->
        <IconButton
          title="Rotate counterclockwise"
          class="material-icons icon-btn"
          onclick={rotateImageLeft}
          disabled={!imageDataUrl}
          >
          rotate_90_degrees_ccw
        </IconButton>
        <!-- Rotate right button -->
        <IconButton
          title="Rotate clockwise"
          class="material-icons icon-btn"
          onclick={rotateImageRight}
          disabled={!imageDataUrl}
          >
          rotate_90_degrees_cw
        </IconButton>
      </div>
    </div>
  {/if}

  {#if checkEditStatus(EditState.Cropping)}
    <div class="actions-section">
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

  {#if checkEditStatus(EditState.Resizing)}
    <div class="actions-section">
      <div class="actions-form">
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

  {#if checkEditStatus(EditState.SettingFilters)}
    <div class="titled-actions-section">
      <div class="actions-section filter-btns">
        <div class="actions">
          <button
            class={grayscaling ? "btn action-btn active" : "btn action-btn"}
            onclick={toggleGrayscale}
            >
            GRAYSCALE
          </button>
          <button
            class={sepia ? "btn action-btn active" : "btn action-btn"}
            onclick={toggleSepia}
            >
            SEPIA
          </button>
        </div>
      </div>
    </div>
  {/if}

  {#if grayscaling}
    <div class="titled-actions-section">
      <div class="actions-section-header">
        Morphology
      </div>
      <div class="actions-section filter-btns">
        <div class="actions">
          <button
            class={grayscaleMorph === "dilate" ? "btn action-btn active" : "btn action-btn"}
            onclick={() => setGrayscaleMorph("dilate")}
            >
            DILATE
          </button>
          <button
            class={grayscaleMorph === "erode" ? "btn action-btn active" : "btn action-btn"}
            onclick={() => setGrayscaleMorph("erode")}
            >
            ERODE
          </button>
          <button
            class={grayscaleMorph === "open" ? "btn action-btn active" : "btn action-btn"}
            onclick={() => setGrayscaleMorph("open")}
            >
            OPEN
          </button>
          <button
            class={grayscaleMorph === "close" ? "btn action-btn active" : "btn action-btn"}
            onclick={() => setGrayscaleMorph("close")}
            >
            CLOSE
          </button>
        </div>
      </div>
    </div>
    {#if grayscaleApplied}
      <div class="titled-actions-section">
        <div class="actions-section-header">
          Mask
        </div>
        <div class="actions-section filter-btns">
          <div class="actions">
            <button
              class={grayscaleMask === "square" ? "btn action-btn active" : "btn action-btn"}
              onclick={() => setGrayscaleMask("square")}
              >
              SQUARE
            </button>
            <button
              class={grayscaleMask === "disk" ? "btn action-btn active" : "btn action-btn"}
              onclick={() => setGrayscaleMask("disk")}
              >
              DISK
            </button>
            <button
              class={grayscaleMask === "diamond" ? "btn action-btn active" : "btn action-btn"}
              onclick={() => setGrayscaleMask("diamond")}
              >
              DIAMOND
            </button>
          </div>
        </div>
      </div>
      <div class="titled-actions-section">
        <div class="actions-section-header">
          Mask Radius
        </div>
        <div class="actions-section filter-btns">
          <input
            type="number"
            name="mask-radius"
            inputmode="numeric"
            min="1"
            max="9"
            bind:value={grayscaleRadius}
            autofocus
          />
        </div>
      </div>
    {/if}
  {/if}

  {#if checkEditStatus(EditState.Cropping) || checkEditStatus(EditState.Resizing) || sepia || grayscaleApplied || rotateApplied}
    <div class="actions-section">
      <div class="actions">
        <!-- Apply edits button -->
        <IconButton
          title="Apply edits"
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
          onclick={resetEdits}
          disabled={!imageDataUrl}
          >
          cancel
        </IconButton>
      </div>
    </div>
  {/if}
</div>

<style>
  input {
    all: unset;
    color: ghostwhite;
    font-style: oblique;
  }

  .actions-section {
    padding: 2px 3px;
    margin: 3px;
  }

  .actions-section.filter-btns {
    display: flex;
    justify-content: center;
  }

  .actions-section-header {
    font-size: 12px;
    color: var(--im-label);
    align-self: anchor-center;
  }

  .titled-actions-section {
    display: flex;
    margin: 3px;
    flex-direction: column;
    justify-content: center;
  }

  .actions {
    justify-content: flex-start;
    display: flex;
    gap: 8px;
  }

  .action-btn {
    width: auto;
    height: 25px;
    padding: 0 4px 0 4px;
    font-size: 12px;
    font-weight: normal;
  }

  .action-btn.active {
    background: var(--im-btn-active-gold);
    border: 1px solid var(--im-btn-active-gold);
  }

  .actions-form {
    display: flex;
    justify-content: center;
    gap: 8px;
  }

  .form-row span {
    width: auto;
    color: var(--im-label);
    font-size: 14px;
  }

  .form-row input {
    border-bottom: 1px solid transparent;
    font-size: 14px;
    width: 4rem;
    cursor: text;
  }

  .form-row input:focus {
    border-bottom: 1px solid ghostwhite;
  }

  input[name="mask-radius"] {
    width: 30px;
  }

  @media (max-width: 640px) {
  }
</style>
