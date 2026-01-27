<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import IconButton from "@smui/icon-button";
  import { currentUser } from "../store.ts";
  import { getCurrentUser, imageUploadUrl } from "../utils/api.ts";

  const dispatch = createEventDispatcher();

  let selectedFiles: File[] = $state([]);
  let previewUrls: string[] = $state([]);
  let uploading: boolean = $state(false);
  let message: string = $state("");
  let messageType: "success" | "error" | "" = $state("");

  async function userIsValid(): boolean {
    const user = await getCurrentUser();
    if (user != null && user === $currentUser) {
      return true;
    }
    return false;
  }

  function readFile(file: File): Promise {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();

      reader.onload = (e) => {
        resolve(e.target?.result as string);
      };

      reader.onerror = () => {
        reject(reader.error);
      };

      reader.readAsDataURL(file);
    });
  }

  async function handleFileSelect(event: Event) {
    const input = event.target as HTMLInputElement;
    if (input.files && input.files.length) {
      selectedFiles = input.files;

      const filePromises = [];
      for (const file of input.files) {
        filePromises.push(readFile(file));
      }

      try {
        // Create preview image(s)
        const results = await Promise.all(filePromises);
        previewUrls = results;
      } catch (error) {
        showMessage(`Failed to create preview image(s): ${error}`, "error");
        console.error("Preview image error:", error);
      }

      // Clear previous messages
      message = "";
      messageType = "";
    }
  }

  async function uploadImage() {
    if (!selectedFiles.length) {
      showMessage("Please select one or more image(s) first", "error");
      return;
    }

    uploading = true;
    message = "";

    try {
      const userValid = await userIsValid();
      if (!userValid) {
        return new Error("Couldn't fetch user session info");
      }

      // Create FormData and append the image
      const formData = new FormData();
      formData.append("user", $currentUser);

      Array.from(selectedFiles).forEach((file) => {
        formData.append("files[]", file, file.name);
      });

      // Send multipart form data request
      const response = await fetch(imageUploadUrl(), {
        method: "POST",
        body: formData,
      });

      if (response.ok) {
        showMessage("Image(s) uploaded successfully", "success");
        dispatch("uploadSuccess");

        // Reset form after successful upload
        setTimeout(() => {
          resetForm();
        }, 2000);
      } else {
        const data = await response.json();
        showMessage(`❌ ${data.error}`, "error");
      }
    } catch (error) {
      showMessage(`❌ Upload failed: ${error}`, "error");
      console.error("Upload error:", error);
    } finally {
      uploading = false;
    }
  }

  function showMessage(msg: string, type: "success" | "error") {
    message = msg;
    messageType = type;
  }

  function resetForm() {
    selectedFiles = [];
    previewUrls = [];
    message = "";
    messageType = "";

    // Reset file input
    const fileInput = document.querySelector("input[type='file']") as HTMLInputElement;
    if (fileInput) fileInput.value = "";
  }

  function close() {
    const modal = document.getElementById("upload-backdrop");
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
</script>

<div
  class="modal-backdrop"
  id="upload-backdrop"
  onclick={handleBackdropClick}
  >
  <div class="modal-content">
    <div class="inner">
      <IconButton
        class="material-icons icon-btn close-btn"
        onclick={close}
        aria-label="Close"
        >
        close
      </IconButton>

      <div class="upload-section">
        <h2>Upload Image(s)</h2>
        <label
          for="file-upload"
          class="file-upload-label"
          disabled={uploading}
          >
          Browse Device
        </label>
        <input
          id="file-upload"
          type="file"
          name="files[]"
          accept="image/jpeg,image/png,image/gif,image/webp,image/bmp"
          onchange={handleFileSelect}
          disabled={uploading}
          multiple
        />

        {#if previewUrls.length}
          <div class="preview">
            {#each previewUrls as imageUrl}
              <img src={imageUrl} alt="Preview" />
            {/each}
          </div>
        {/if}

        <button
          class="btn"
          onclick={uploadImage}
          disabled={!selectedFiles.length || uploading}
          >
          {uploading ? "Uploading..." : "Upload"}
        </button>

        {#if message}
          <div class="message {messageType}">
            {message}
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .upload-section {
    width: 100%;
    height: auto;
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 300px;
    flex-direction: column;
    gap: 20px;
  }

  input[type="file"] {
    display: none;
    padding: 12px;
    position: absolute;
    cursor: pointer;
  }

  .file-upload-label {
    display: inline-block;
    padding: 12px;
    border: 2px dashed #667eea;
    border-radius: 8px;
    cursor: pointer;
    background: none;
    transition: border-color 0.2s;
  }

  .file-upload-label:hover:not(:disabled) {
    border-color: #764ba2;
  }

  .file-upload-label:disabled {
    cursor: not-allowed;
  }

  .preview {
    text-align: center;
  }

  .preview img {
    max-width: 100%;
    max-height: 200px;
  }

  .message {
    padding: 12px;
    color: var(--im-text);
    text-align: center;
    font-weight: 500;
  }

  .message.error {
    background: var(--im-warn);
  }
</style>
