<script lang="ts">
  import Header from "./components/Header.svelte";
  import Footer from "./components/Footer.svelte";
  import UploadForm from "./components/UploadForm.svelte";
  import ImageGallery from "./components/ImageGallery.svelte";
  import ImageViewer from "./components/ImageViewer.svelte";
  import UserRegister from "./components/UserRegister.svelte";
  import UserLogin from "./components/UserLogin.svelte";

  import { currentView } from "./store.ts";
  $currentView = "login";

  export interface ImageData {
    key: string;
    size: number;
    last_modified: string;
    content_type: string;
  }

  let selectedImage: ImageData | null = null;
  let refreshTrigger = 0;

  function handleImageSelect(event: CustomEvent<ImageData>) {
    selectedImage = event.detail;
  }

  function handleImageClose() {
    selectedImage = null;
  }

  function handleUploadSuccess() {
    // Trigger gallery refresh
    refreshTrigger++;
    selectedImage = null;
  }

  function handleLoginSuccess() {
    $currentView = "upload";
  }

  function handleShowRegisterView() {
    $currentView = "register";
  }

  function handleRegistrationSuccess() {
    $currentView = "login";
  }
</script>

<div class="app">
  <Header />

  <main>
    <div class="container">
      {#if $currentView === "upload"}
        <UploadForm on:uploadSuccess={handleUploadSuccess} />
      {:else if $currentView === "gallery"}
        <ImageGallery
          on:imageSelect={handleImageSelect}
          refresh={refreshTrigger}
        />

        {#if selectedImage}
          <ImageViewer
            image={selectedImage}
            on:close={handleImageClose}
          />
        {/if}
      {:else if $currentView === "register"}
        <UserRegister
          on:registrationSuccess={handleRegistrationSuccess}
        />
      {:else if $currentView === "login"}
        <UserLogin
          on:loginSuccess={handleLoginSuccess}
          on:showRegisterView={handleShowRegisterView}
        />
      {/if}
    </div>
  </main>

  <Footer />
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
    background: #f5f7fa;
  }

  .app {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
  }

  main {
    flex: 1;
    padding: 32px 20px;
  }

  .container {
    max-width: 1200px;
    margin: 0 auto;
  }
</style>
