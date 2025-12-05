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
        showMessage("✅ Image uploaded successfully", "success");
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

    <IconButton
      class="material-icons close-btn"
      onclick={close}
      aria-label="Close"
      >
      close
    </IconButton>

    <div class="upload-section">
      <h2>Upload New Image</h2>
      <label for="file-upload" class="file-upload-label">
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
        onclick={uploadImage}
        disabled={!selectedFile || uploading}
        class="upload-btn"
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

<style>
  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.85);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 20px;
    animation: fadeIn 0.2s ease-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .modal-content {
    background: white;
    border-radius: 16px;
    max-width: 900px;
    width: 100%;
    max-height: 90vh;
    overflow-y: auto;
    position: relative;
    animation: slideUp 0.3s ease-out;
  }

  @keyframes slideUp {
    from {
      transform: translateY(20px);
      opacity: 0;
    }
    to {
      transform: translateY(0);
      opacity: 1;
    }
  }

  :global(.close-btn) {
    position: absolute;
    top: 16px;
    right: 16px;
    width: 40px;
    height: 40px;
    border-radius: 50%;
    background: rgba(0, 0, 0, 0.5);
    color: white;
    border: none;
    font-size: 24px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10;
    transition: background 0.2s;
  }

  :global(.close-btn:hover) {
    background: rgba(0, 0, 0, 0.7);
  }

  h2 {
    margin: 0 0 24px 0;
    color: #333;
    font-size: 24px;
  }

  .upload-section {
    width: 100%;
    height: 500px;
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 300px;
    flex-direction: column;
    gap: 20px;
  }

  input[type="file"] {
    opacity: 0;
    position: absolute;
    cursor: pointer;
  }

  .file-upload-label {
    display: inline-block;
    padding: 12px;
    border: 2px dashed #667eea;
    border-radius: 8px;
    cursor: pointer;
    background: #f8f9ff;
    transition: border-color 0.2s;
  }

  input[type="file"]::file-selector-button {
    display: none;
  }

  input[type="file"]:hover:not(:disabled) {
    border-color: #764ba2;
  }

  input[type="file"]:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .preview {
    text-align: center;
  }

  .preview img {
    max-width: 100%;
    max-height: 200px;
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }

  .upload-btn {
    padding: 14px 24px;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    border: none;
    border-radius: 8px;
    font-size: 16px;
    font-weight: 600;
    cursor: pointer;
    transition: transform 0.2s, box-shadow 0.2s;
  }

  .upload-btn:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 8px 16px rgba(102, 126, 234, 0.4);
  }

  .upload-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    transform: none;
  }

  .message {
    padding: 12px;
    border-radius: 8px;
    text-align: center;
    font-weight: 500;
  }

  .message.success {
    background: #d4edda;
    color: #155724;
    border: 1px solid #c3e6cb;
  }

  .message.error {
    background: #f8d7da;
    color: #721c24;
    border: 1px solid #f5c6cb;
  }
</style>
