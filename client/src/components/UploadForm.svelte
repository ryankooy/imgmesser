<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { apiUrl, currentUser, getCurrentUser } from "../store.ts";

  const dispatch = createEventDispatcher();

  let selectedFile: File | null = null;
  let previewUrl: string | null = null;
  let uploading = false;
  let message = "";
  let messageType: "success" | "error" | "" = "";

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

      // Send multipart form data request
      const response = await fetch(`${apiUrl}/images`, {
        method: "POST",
        body: formData,
      });

      const data = await response.json();

      if (response.ok && data.success) {
        showMessage(`✅ ${data.message}`, "success");
        dispatch("uploadSuccess");

        // Reset form after successful upload
        setTimeout(() => {
          resetForm();
        }, 2000);
      } else {
        showMessage(`❌ ${data.message}`, "error");
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
</script>

<div class="upload-card" id="upload">
  <h2>Upload New Image</h2>

  <div class="upload-section">
    <input
      type="file"
      accept="image/jpeg,image/png,image/gif,image/webp"
      on:change={handleFileSelect}
      disabled={uploading}
    />

    {#if previewUrl}
      <div class="preview">
        <img src={previewUrl} alt="Preview" />
      </div>
    {/if}

    <button
      on:click={uploadImage}
      disabled={!selectedFile || uploading}
      class="upload-btn"
    >
      {uploading ? "Uploading..." : "Upload Image"}
    </button>

    {#if message}
      <div class="message {messageType}">
        {message}
      </div>
    {/if}
  </div>
</div>

<style>
  .upload-card {
    background: white;
    border-radius: 16px;
    padding: 32px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }

  h2 {
    margin: 0 0 24px 0;
    color: #333;
    font-size: 24px;
  }

  .upload-section {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  input[type="file"] {
    padding: 12px;
    border: 2px dashed #667eea;
    border-radius: 8px;
    cursor: pointer;
    background: #f8f9ff;
    transition: border-color 0.2s;
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
