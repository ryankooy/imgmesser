<script lang="ts">
  import { onMount } from "svelte";
  import IconButton from "@smui/icon-button";
  import { imageUrl } from "../utils/api.ts";
  import { getBytesFromDataUrl, getImageBytes, toggleButtonColor } from "../utils/app.ts";
  import { EditStatus, ImageStatus } from "../store.ts";
  import init, {
    adjust_lighting, apply_grayscale_filter, apply_sepia_filter,
  } from "../pkg/transformjs.js";

  let {
    editStatus = $bindable(),
    height = 0,
    imageDataUrl = "",
    imageId = null,
    status = $bindable(),
    transformations = {},
    transformMenuOpen = false,
    width = 0,
    clearTransformations,
    imageUpdated,
    resetCrop,
    setAlertMessage,
    setAnimatedRotation,
    setAspect,
  } = $props();

  let wasmInitialized: boolean = $state(false);

  onMount(async () => {
    await init();
    wasmInitialized = true;
  });

  let tempImageDataUrl: string | null = $state(null);

  let rotation: number = $state(0);
  let aRotation: number = $state(0);
  let rotateApplied: boolean = $state(false);

  let resizeWidth: number = $derived(width);
  let resizeHeight: number = $derived(height);
  let resizeApplied: boolean = $state(false);

  let settingFilters: boolean = $state(false);
  let grayscaling: boolean = $state(false);
  let grayscaleApplied: boolean = $state(false);

  let grayscaleMorph: string | null = $state(null);
  let grayscaleRadius: number = $state(1);

  let sepia: boolean = $state(false);

  let adjustingBrightness: boolean = $state(false);
  let brightnessApplied: boolean = $state(false);
  let brightnessValue: number = $state(0);

  async function transformImage() {
    if (Object.keys(transformations).length === 0) return;

    try {
      const response = await fetch(`${imageUrl(imageId)}/transform`, {
        method: "POST",
        headers: {"Content-Type": "application/json"},
        body: JSON.stringify(transformations),
      });

      if (response.ok) {
        await imageUpdated();
        resetEdits();
      } else {
        setAlertMessage("Failed to transform image");
      }
    } catch (error) {
      console.error("Error fetching:", error);
    }
  }

  function resetImageDataUrl() {
    const img = document.getElementById("image") as HTMLImageElement;
    if (img) img.src = imageDataUrl;
  }

  function toggleBrightness(node: PointerEvent) {
    adjustingBrightness = !adjustingBrightness;
    toggleButtonColor(node, adjustingBrightness);
  }

  async function rotateImageRight() {
    editStatus.set(EditStatus.Rotating);

    if (rotation === 270) {
      if (!!transformations.rotate) delete transformations.rotate;
      rotation = 0;
      rotateApplied = false;
    } else {
      rotation += 90;
      transformations.rotate = rotation;
      rotateApplied = true;
    }

    aRotation += 90;
    setAnimatedRotation(aRotation);
  }

  async function rotateImageLeft() {
    editStatus.set(EditStatus.Rotating);

    if (rotation === 90) {
      if (!!transformations.rotate) delete transformations.rotate;
      rotation = 0;
      rotateApplied = false;
    } else {
      if (rotation === 0)
        rotation = 270;
      else
        rotation -= 90;

      transformations.rotate = rotation;
      rotateApplied = true;
    }

    aRotation -= 90;
    setAnimatedRotation(aRotation);
  }

  async function applyEdits() {
    status.set(ImageStatus.Loading);

    if (grayscaling) grayscaleImage();

    await transformImage();
  }

  function resetEdits() {
    if (adjustingLighting()) resetLighting();

    clearTransformations();

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
    resetLighting();

    editStatus.reset();
    status.reset();
  }

  function resetRotate() {
    rotateApplied = false;
    rotation = aRotation = 0;
    setAnimatedRotation(0);
  }

  function resizeImage() {
    if (resizeWidth !== width || resizeHeight !== height) {
      transformations.resize = { width: resizeWidth, height: resizeHeight };
      resizeApplied = true;
    } else {
      if (!!transformations.resize) delete transformations.resize;
      if (resizeApplied) resizeApplied = false;
    }
  }

  function resetResize() {
    resizeApplied = false;
    resizeWidth = width;
    resizeHeight = height;
  }

  function toggleGrayscale() {
    grayscaling = !grayscaling;
    transformations.filters ??= {};
    transformations.filters.grayscale = grayscaling;

    if (grayscaling && sepia) {
      sepia = false;
      transformations.filters.sepia = false;
      resetImageDataUrl();
    } else if (!grayscaling) {
      resetGrayscale();
      resetImageDataUrl();
    }
  }

  function grayscaleImage() {
    if (!grayscaleMorph) return;

    let grayscaleOptions: object = {
      morphology: grayscaleMorph,
      mask_radius: grayscaleRadius,
    };

    transformations.filters.options ??= {};
    Object.assign(transformations.filters.options, grayscaleOptions);
  }

  function resetGrayscale() {
    grayscaling = grayscaleApplied = false;
    grayscaleMorph = null;
    grayscaleRadius = 1;
  }

  function setGrayscale(morphology: string) {
    grayscaleMorph = morphology;
    grayscaleApplied = true;

    if (wasmInitialized) {
      rerenderImage((bytes) => {
        return apply_grayscale_filter(bytes, width, height, {
          morphology: grayscaleMorph,
          mask_radius: grayscaleRadius,
        });
      });
    }
  }

  function toggleSepia() {
    sepia = !sepia;
    transformations.filters ??= {};
    transformations.filters.sepia = sepia;

    if (sepia && wasmInitialized) {
      resetGrayscale();
      transformations.filters.grayscale = false;

      rerenderImage((bytes) => {
        return apply_sepia_filter(bytes, width, height);
      });
    } else if (!sepia) {
      resetImageDataUrl();
    }
  }

  function resetFilters() {
    resetGrayscale();
    sepia = false;
  }

  function applyBrightness() {
    transformations.lighting ??= {};
    if (brightnessValue === 0) {
      if (!!transformations.lighting.brightness)
        delete transformations.lighting.brightness;
      brightnessApplied = false;
    } else {
      transformations.lighting.brightness = brightnessValue;
      brightnessApplied = true;
    }

    if (wasmInitialized) {
      rerenderImage((bytes) => {
        return adjust_lighting(bytes, width, height, {
          brightness: brightnessValue,
        });
      });
    }
  }

  async function rerenderImage(f: (bytes: Uint8ClampedArray) => Uint8ClampedArray) {
    const [img, canvas, ctx] = makeCanvas();

    const bytes = await getBytesFromDataUrl(imageDataUrl);

    // Pass pixels to Wasm via given function
    const pixels: Uint8ClampedArray = f(bytes);

    renderCanvasPixels(img, canvas, ctx, pixels);
  }

  function makeCanvas() {
    const img = document.getElementById("image") as HTMLImageElement | null;
    if (!img) return;

    // Create offscreen canvas
    const canvas = document.createElement("canvas");
    const ctx = canvas.getContext("2d");

    if (!ctx) {
      console.error("2D context not supported");
      return;
    }

    canvas.width = width;
    canvas.height = height;

    return [img, canvas, ctx];
  }

  function renderCanvasPixels(img, canvas, ctx, pixels) {
    const newImageData = new ImageData(
      new Uint8ClampedArray(pixels),
      width,
      height,
    );

    // Render processed pixels back to image
    ctx.putImageData(newImageData, 0, 0);

    tempImageDataUrl = canvas.toDataURL();
    img.src = tempImageDataUrl;
  }

  function resetLighting() {
    adjustingBrightness = brightnessApplied = false;
    resetBrightness();
  }

  function adjustingLighting(): boolean {
    return adjustingBrightness;
  }

  function resetBrightness() {
    brightnessValue = 0;
    resetSliderElement();
    resetImageDataUrl();
  }

  function handleWidthInput(event: Event) {
    const target = event.currentTarget as HTMLInputElement;
    resizeWidth = parseInt(target.value, 10);
    resizeHeight = Math.round((resizeWidth * height) / width);
    resizeImage();
  }

  function handleHeightInput(event: Event) {
    const target = event.currentTarget as HTMLInputElement;
    resizeHeight = parseInt(target.value, 10);
    resizeWidth = Math.round((resizeHeight * width) / height);
    resizeImage();
  }

  function transformApplied(): boolean {
    return (
      sepia || grayscaleApplied || resizeApplied || rotateApplied ||
      editStatus.check(EditStatus.Cropping) || brightnessApplied
    );
  }

  function updateSlider(node: HTMLElement) {
    const handleScroll = (event) => {
      const slider = event.target;
      const min = slider.min || -100;
      const max = slider.max || 100;
      const percent = ((slider.value - min) / (max - min)) * 100;
      slider.style.setProperty("--val", `${percent}%`);
    }

    node.addEventListener("input", handleScroll);

    return {
      destroy() {
        node.removeEventListener("input", handleScroll);
      }
    };
  }

  function resetSliderElement() {
    const slider = document.getElementsByClassName("slider")[0];
    if (slider) slider.style.setProperty("--val", "50%");
  }

  resetSliderElement();
</script>

{#if !transformMenuOpen}
  <div
    in:fade={{ duration: 200 }}
    out:fade={{ duration: 200 }}
    >
    {#if editStatus.check(EditStatus.Rotating)}
      <div class="titled-actions-section">
        <div class="actions-section-header">
          ROTATE
        </div>
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
      </div>
    {/if}

    {#if editStatus.check(EditStatus.Cropping)}
      <div class="titled-actions-section">
        <div class="actions-section-header">
          CROP
        </div>
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
      </div>
    {/if}

    {#if editStatus.check(EditStatus.Resizing)}
      <div class="titled-actions-section">
        <div class="actions-section-header">
          RESIZE
        </div>
        <div class="actions-section">
          <div class="actions-form">
            <label class="form-row">
              <span>Width</span>
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
              <span>Height</span>
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
      </div>
    {/if}

    {#if editStatus.check(EditStatus.SettingFilters)}
      <div class="titled-actions-section">
        <div class="actions-section-header">
          FILTERS
        </div>
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

    {#if editStatus.check(EditStatus.EditingLighting)}
      <div class="titled-actions-section">
        <div class="actions-section-header">
          LIGHTING
        </div>
        <div class="actions-section lighting-btns">
          <div class="actions">
            <IconButton
              title="Brightness"
              class="material-icons icon-btn"
              onclick={toggleBrightness}
              disabled={!imageDataUrl}
              >
              brightness_medium
            </IconButton>
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
              onclick={() => setGrayscale("dilate")}
              >
              DILATE
            </button>
            <button
              class={grayscaleMorph === "erode" ? "btn action-btn active" : "btn action-btn"}
              onclick={() => setGrayscale("erode")}
              >
              ERODE
            </button>
            <button
              class={grayscaleMorph === "open" ? "btn action-btn active" : "btn action-btn"}
              onclick={() => setGrayscale("open")}
              >
              OPEN
            </button>
            <button
              class={grayscaleMorph === "close" ? "btn action-btn active" : "btn action-btn"}
              onclick={() => setGrayscale("close")}
              >
              CLOSE
            </button>
          </div>
        </div>
      </div>

      {#if grayscaleApplied}
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
              oninput={() => setGrayscale(grayscaleMorph)}
              autofocus
            />
          </div>
        </div>
      {/if}
    {/if}

    {#if adjustingBrightness}
      <div class="titled-actions-section">
        <div class="actions-section-header">
          Brightness
        </div>
      </div>
      <div class="actions-section">
        <div class="actions">
          <div class="slider-container">
            <input
              use:updateSlider
              name="brightness"
              class="slider"
              type="range"
              min="-100"
              max="100"
              step="10"
              bind:value={brightnessValue}
              oninput={applyBrightness}
            />
          </div>
        </div>
      </div>
    {/if}

    {#if !editStatus.check(EditStatus.None)}
      <div class="actions-section">
        <div class="actions">
          <!-- Apply edits button -->
          <IconButton
            title="Apply edits"
            class="material-icons icon-btn"
            onclick={applyEdits}
            disabled={!imageDataUrl || !transformApplied()}
            >
            check
          </IconButton>

          {#if adjustingBrightness}
            <!-- Reset edits button -->
            <IconButton
              title="Reset edits"
              class="material-icons icon-btn"
              onclick={resetBrightness}
              disabled={!imageDataUrl}
              >
              refresh
            </IconButton>
          {/if}

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
{/if}

<style>
  input[type="number"] {
    all: unset;
    color: ghostwhite;
    font-style: oblique;
  }

  .filter-btns {
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

  .form-row {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-direction: column;
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
    text-align: center;
  }

  .form-row input:focus {
    border-bottom: 1px solid ghostwhite;
  }

  input[name="mask-radius"] {
    width: 30px;
  }

  .slider-container {
    width: 100%;
    max-width: 300px;
    margin: 1rem auto;
    --track-bg: #ddd;
    --fill-bg: var(--im-hover-gold);
    --thumb-bg: ghostwhite;
    --thumb-size: 20px;
  }

  input[type="range"] {
    -webkit-appearance: none;
    appearance: none;
    width: 100%;
    height: 8px;
    cursor: pointer;
    border-radius: 4px;
  }

  /* Slider track */
  input[type="range"]::-webkit-slider-runnable-track {
    width: 100%;
    height: 8px;
    border-radius: 4px;
    outline: none;
    background: linear-gradient(
      to right,
      var(--track-bg) 0%,
      var(--track-bg) min(var(--val), 50%),
      var(--fill-bg) min(var(--val), 50%),
      var(--fill-bg) max(var(--val), 50%),
      var(--track-bg) max(var(--val), 50%),
      var(--track-bg) 100%
    );
  }

  input[type="range"]::-moz-range-track {
    width: 100%;
    height: 8px;
    border-radius: 4px;
    outline: none;
    background: linear-gradient(
      to right,
      var(--track-bg) 0%,
      var(--track-bg) min(var(--val), 50%),
      var(--fill-bg) min(var(--val), 50%),
      var(--fill-bg) max(var(--val), 50%),
      var(--track-bg) max(var(--val), 50%),
      var(--track-bg) 100%
    );
  }

  /* Slider thumb */
  input[type=range]::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    margin-top: -6px; /* Align center of thumb with center of track */
    height: var(--thumb-size);
    width: var(--thumb-size);
    border-radius: 50%;
    background: var(--thumb-bg);
    cursor: pointer;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.8);
    transition: background 0.15s ease-in-out;
  }
  input[type=range]::-moz-range-thumb {
    border: none;
    height: var(--thumb-size);
    width: var(--thumb-size);
    border-radius: 50%;
    background: var(--thumb-bg);
    cursor: pointer;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.8);
    transition: background 0.15s ease-in-out;
  }
  input[type="range"]:active::-webkit-slider-thumb {
    transform: scale(1.1);    /* Slightly enlarges the knob */
  }
  input[type="range"]:active::-moz-range-thumb {
    transform: scale(1.1);
  }

  @media (max-width: 640px) {
  }
</style>
