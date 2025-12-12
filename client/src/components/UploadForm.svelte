<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import IconButton from "@smui/icon-button";
  import { apiUrl, currentUser, getCurrentUser } from "../store.ts";

  const dispatch = createEventDispatcher();

  let selectedFile: File | null = $state(null);
  let previewUrl: string | null = $state(null);
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

  function handleFileSelect(event: Event) {
    const input = event.target as HTMLInputElement;
    if (input.files && input.files[0]) {
      selectedFile = input.files[0];

      // Create preview
      const reader = new FileReader();
      reader.onload = (e) => {
        previewUrl = e.target?.result as string;
      };
      reader.readAsDataURL(selectedFile);

      // Clear previous messages
      message = "";
      messageType = "";
    }
  }

  async function uploadImage() {
    if (!selectedFile) {
      showMessage("Please select an image first", "error");
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
      formData.append("file_path", selectedFile);
      formData.append("user", $currentUser);

      // Send multipart form data request
      const response = await fetch(`${apiUrl}/images`, {
        method: "POST",
        body: formData,
      });

      if (response.ok) {
        showMessage("Image uploaded successfully", "success");
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
    selectedFile = null;
    previewUrl = null;
    message = "";
    messageType = "";

    // Reset file input
    const fileInput = document.querySelector("input[type='file']") as HTMLInputElement;
    if (fileInput) fileInput.value = "";
  }

  function close() {
    dispatch("close");
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      close();
    }
  }
</script>

<div class="modal-backdrop" onclick={handleBackdropClick}>
  <div class="modal-content" id="upload">
    <div class="inner">
      <IconButton
        class="material-icons icon-btn close-btn"
        onclick={close}
        aria-label="Close"
        >
        close
      </IconButton>

      <div class="upload-section">
        <h2>Upload New Image</h2>
        <label
          for="file-upload"
          class="file-upload-label"
          disabled={uploading}
          >
          Browse Computer
        </label>
        <input
          id="file-upload"
          type="file"
          accept="image/jpeg,image/png,image/gif,image/webp"
          onchange={handleFileSelect}
          disabled={uploading}
        />

        {#if previewUrl}
          <div class="preview">
            <img src={previewUrl} alt="Preview" />
          </div>
        {/if}

        <button
          class="btn"
          onclick={uploadImage}
          disabled={!selectedFile || uploading}
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
    background: darkred;
  }
</style>
