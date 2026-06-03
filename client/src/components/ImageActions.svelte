<script lang="ts">
  import IconButton from "@smui/icon-button";
  import { EditState, ImageState, imageDataUrlCache } from "../store.ts";
  import { imageUrl } from "../utils/api.ts";

  let {
    imageDataUrl = "",
    imageId = null,
    meta = null,
    checkStatus,
    closeImage,
    downloadImage,
    handleDeleteImage,
    handleDiscardEdits,
    handleUpdatedImage,
    resetEditStatus,
    saveImageEdits,
    setAlertMessage,
    toggleButtonColor,
    toggleStatus,
  } = $props();

  const multiVersion: boolean = $derived(meta.version_count > 1);

  async function undoEdit() {
    try {
      const response = await fetch(`${imageUrl(imageId)}/revert`, {
        method: "POST",
      });

      if (response.ok) {
        const data = await response.json();
        if (data.updated) {
          $imageDataUrlCache.delete(imageId);
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
      const response = await fetch(`${imageUrl(imageId)}/restore`, {
        method: "POST",
      });

      if (response.ok) {
        const data = await response.json();
        if (data.updated) {
          $imageDataUrlCache.delete(imageId);
          await handleUpdatedImage();
        }
      } else {
        setAlertMessage("Failed to restore image");
      }
    } catch (error) {
      console.error("Error fetching:", error);
    }
  }

  function typeIsGif(): boolean {
    return meta.content_type === "image/gif";
  }

  function toggleTransform(node: PointerEvent) {
    if (checkStatus(ImageState.Panning)) {
      const panBtn = document.querySelector(".toggle-btn.pan-btn") as HTMLElement;
      panBtn.style.color = "white";
    }

    toggleStatus(ImageState.Transforming);
    toggleButtonColor(node, checkStatus(ImageState.Transforming));

    if (!checkStatus(ImageState.Transforming)) resetEditStatus();
  }

  function togglePanTool(node: PointerEvent) {
    toggleStatus(ImageState.Panning);
    toggleButtonColor(node, checkStatus(ImageState.Panning));
  }
</script>

<div>
  <div class="actions-section">
    <div class="actions">
      <!-- Transform button -->
      <IconButton
        title="Toggle transform tools"
        class="material-icons icon-btn toggle-btn transform-btn"
        onclick={toggleTransform}
        disabled={!imageDataUrl || checkStatus(ImageState.Panning) || typeIsGif()}
        >
        transform
      </IconButton>
      <!-- Pan tool button -->
      <IconButton
        title="Pan tool"
        class="material-icons icon-btn toggle-btn pan-btn"
        onclick={togglePanTool}
        disabled={!imageDataUrl || checkStatus(ImageState.Transforming)}
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
        onclick={closeImage}
        aria-label="Close"
        >
        close
      </IconButton>
    </div>
  </div>

  {#if multiVersion && !typeIsGif()}
    <div class="actions-section">
      <div class="actions">
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
</div>

<style>
  .actions-section {
    padding: 2px 3px;
    margin: 3px;
  }

  .actions {
    justify-content: flex-start;
    display: flex;
    gap: 8px;
  }

  :global(.delete-btn:hover:not(:disabled)) {
    background: var(--im-warn);
  }

  :global(.delete-btn:active:not(:disabled)) {
    background: var(--im-btn-active-warn);
  }

  @media (max-width: 640px) {
  }
</style>
