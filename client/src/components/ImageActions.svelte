<script lang="ts">
  import IconButton from "@smui/icon-button";
  import { EditState, ImageState, imageDataUrlCache } from "../store.ts";
  import { imageUrl } from "../utils/api.ts";
  import DropdownMenu from "./DropdownMenu.svelte";

  let {
    imageDataUrl = "",
    imageId = null,
    meta = null,
    checkStatus,
    closeImage,
    deleteImage,
    downloadImage,
    discardAllEdits,
    discardCurrentEdit,
    imageUpdated,
    resetEditStatus,
    saveImage,
    setAlertMessage,
    toggleButtonColor,
    toggleStatus,
  } = $props();

  interface MenuItem {
    title: string;
    iconName: string;
    func: (() => void) | null;
  }

  interface Menu {
    title: string;
    iconName: string;
    items: MenuItem[];
  }

  const multiVersion: boolean = $derived(meta.version_count > 1);

  const saveMenu: Menu = {
    title: "Save options",
    iconName: "save",
    items: [
      {
        title: "Save current edit",
        iconName: "save",
        func: () => saveImage(),
      },
      {
        title: "Save current edit as...",
        iconName: "save_as",
        func: null,
      },
    ],
  };

  const discardMenu: Menu = {
    title: "Discard options",
    iconName: "delete",
    items: [
      {
        title: "Discard current edit",
        iconName: "delete",
        func: () => discardCurrentEdit(),
      },
      {
        title: "Discard all edits",
        iconName: "delete_sweep",
        func: () => discardAllEdits(),
      },
    ],
  };

  async function undoEdit() {
    try {
      const response = await fetch(`${imageUrl(imageId)}/revert`, {
        method: "POST",
      });

      if (response.ok) {
        const data = await response.json();
        if (data.updated) {
          $imageDataUrlCache.delete(imageId);
          await imageUpdated();
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
          await imageUpdated();
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
      <!-- Pan tool button -->
      <IconButton
        title="Pan tool"
        class="material-icons icon-btn toggle-btn pan-btn"
        onclick={togglePanTool}
        disabled={!imageDataUrl || checkStatus(ImageState.Transforming)}
        >
        pan_tool
      </IconButton>
      <!-- Transform button -->
      <IconButton
        title="Toggle transform tools"
        class="material-icons icon-btn toggle-btn transform-btn"
        onclick={toggleTransform}
        disabled={!imageDataUrl || checkStatus(ImageState.Panning) || typeIsGif()}
        >
        transform
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
        onclick={deleteImage}
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

  {#if multiVersion}
    <div class="actions-section">
      <div class="actions">
        <!-- Unto button -->
        <IconButton
          title="Undo change"
          class="material-icons icon-btn"
          onclick={undoEdit}
          disabled={!imageDataUrl || meta.initial_version}
          >
          undo
        </IconButton>
        <!-- Redo button -->
        <IconButton
          title="Redo change"
          class="material-icons icon-btn"
          onclick={redoEdit}
          disabled={!imageDataUrl || meta.latest_version}
          >
          redo
        </IconButton>

        {#if meta.initial_version}
          <!-- Save-as button -->
          <IconButton
            title="Save image as..."
            class="material-icons icon-btn"
            disabled={!imageDataUrl}
            >
            save_as
          </IconButton>
          <!-- Discard all edits button -->
          <IconButton
            title="Discard all edits"
            class="material-icons icon-btn delete-btn"
            onclick={discardAllEdits}
            disabled={!imageDataUrl}
            >
            delete_sweep
          </IconButton>
        {:else}
          <!-- Save-edit dropdown -->
          <DropdownMenu
            imageDataUrl={imageDataUrl}
            menu={saveMenu}
          />
          <!-- Discard-edit dropdown -->
          <DropdownMenu
            imageDataUrl={imageDataUrl}
            menu={discardMenu}
          />
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  :global(.delete-btn:hover:not(:disabled)) {
    background: var(--im-warn);
  }

  :global(.delete-btn:active:not(:disabled)) {
    background: var(--im-btn-active-warn);
  }

  @media (max-width: 640px) {
  }
</style>
