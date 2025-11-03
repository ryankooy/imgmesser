<script lang="ts">
  let selectedFile: File | null = null;
  let previewUrl: string | null = null;
  let uploading = false;
  let message = "";
  let messageType: "success" | "error" | "" = "";
  let bucketName = "imgmesser-storage";
  let imageUri: string | null = null;

  const API_URL = "http://127.0.0.1:3000";

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
      // Create FormData and append the image
      const formData = new FormData();
      formData.append("file_path", selectedFile);

      // Send multipart form data request
      let response = await fetch(`${API_URL}/images`, {
        method: "POST",
        body: formData,
        // Don't set Content-Type header - browser will set it with boundary
      });

      let data = await response.json();

      if (response.ok && data.success) {
        showMessage(`✅ ${data.message}`, "success");

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

<main>
  <div class="container">
    <h1>ImgMesser</h1>
    <p class="subtitle">Upload an image to mess with (when such functionality is in place)</p>

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
          <p class="filename">{selectedFile?.name}</p>
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
</main>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    min-height: 100vh;
  }

  main {
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 100vh;
    padding: 20px;
  }

  .container {
    background: white;
    border-radius: 16px;
    padding: 40px;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
    max-width: 500px;
    width: 100%;
  }

  h1 {
    margin: 0 0 8px 0;
    color: #333;
    text-align: center;
  }

  .subtitle {
    text-align: center;
    color: #666;
    margin: 0 0 32px 0;
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
    max-height: 300px;
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }

  .filename {
    margin-top: 12px;
    color: #666;
    font-size: 14px;
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
