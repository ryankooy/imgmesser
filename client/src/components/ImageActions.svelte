<script lang="ts">
  import IconButton from "@smui/icon-button";
  import { EditStatus, ImageStatus, imageDataUrlCache } from "../store.ts";
  import type { IconMenu } from "../store.ts";
  import { imageUrl } from "../utils/api.ts";
  import { toggleHidden } from "../utils/app.ts";
  import DropdownMenu from "./DropdownMenu.svelte";

  let {
    editStatus = $bindable(),
    imageDataUrl = "",
    imageId = null,
    meta = null,
    status = $bindable(),
    transformMenuOpen = false,
    closeImage,
    deleteImage,
    downloadImage,
    discardAllEdits,
    discardCurrentEdit,
    imageUpdated,
    saveImage,
    setAlertMessage,
    toggleButtonColor,
  } = $props();

  const multiVersion: boolean = $derived(meta.version_count > 1);

  const transformMenu: IconMenu = {
    title: "Transform tools",
    iconName: "transform",
    handleClick: () => openTransformMenu(),
    handleClickOutside: () => status.reset(),
    toggleFunc: (hide: boolean) => toggleEditIcons(hide),
    items: [
      {
        title: "Rotate",
        iconName: "rotate_left",
        handleClick: () => handleTransform(EditStatus.Rotating),
      },
      {
        title: "Crop",
        iconName: "crop",
        handleClick: () => handleTransform(EditStatus.Cropping),
      },
      {
        title: "Resize",
        iconName: "photo_size_select_small",
        handleClick: () => handleTransform(EditStatus.Resizing),
      },
      {
        title: "Filters",
        iconName: "filter_b_and_w",
        handleClick: () => handleTransform(EditStatus.SettingFilters),
      },
    ],
  };

  const saveMenu: IconMenu = {
    title: "Save options",
    iconName: "save",
    items: [
      {
        title: "Save current edit",
        iconName: "save",
        handleClick: () => saveImage(),
      },
      {
        title: "Save current edit as...",
        iconName: "save_as",
        //TODO: add click handler
      },
    ],
  };

  const discardMenu: IconMenu = {
    title: "Discard options",
    iconName: "delete",
    items: [
      {
        title: "Discard current edit",
        iconName: "delete",
        handleClick: () => discardCurrentEdit(),
      },
      {
        title: "Discard all edits",
        iconName: "delete_sweep",
        handleClick: () => discardAllEdits(),
      },
    ],
  };

  function toggleEditIcons(hide: boolean) {
    const editIcons = document.getElementById("edit-actions") as HTMLElement;
    toggleHidden(editIcons, hide);
  }

  function handleTransform(status: EditStatus) {
    transformMenuOpen = false;
    editStatus.set(status);
  }

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

  function openTransformMenu() {
    if (status.check(ImageStatus.Panning)) {
      const panBtn = document.querySelector(".toggle-btn.pan-btn") as HTMLElement;
      panBtn.style.color = "white";
    }

    status.set(ImageStatus.Transforming);
    transformMenuOpen = true;
    editStatus.reset();
  }

  function togglePanTool(node: PointerEvent) {
    status.toggle(ImageStatus.Panning);

    const isPanning: boolean = status.check(ImageStatus.Panning);

    toggleButtonColor(node, isPanning);
    toggleEditIcons(isPanning);
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
        disabled={!imageDataUrl || status.check(ImageStatus.Transforming)}
        >
        pan_tool
      </IconButton>
      <!-- Transform menu button -->
      <DropdownMenu
        menu={transformMenu}
        disabled={!imageDataUrl || typeIsGif() || !status.check(ImageStatus.None)}
      />
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

  {#if multiVersion && editStatus.check(EditStatus.None)}
    <div class="actions-section fade-element" id="edit-actions">
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
            menu={saveMenu}
            disabled={!imageDataUrl}
          />
          <!-- Discard-edit dropdown -->
          <DropdownMenu
            menu={discardMenu}
            disabled={!imageDataUrl}
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
