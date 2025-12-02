<script lang="ts">
  import { onMount } from "svelte";
  import {
    apiUrl, currentView, currentUser, getCurrentUser, registerServiceWorker,
  } from "./store.ts";
  import type { ImageData } from "./store.ts";

  import Header from "./components/Header.svelte";
  import Footer from "./components/Footer.svelte";
  import UploadForm from "./components/UploadForm.svelte";
  import ImageGallery from "./components/ImageGallery.svelte";
  import ImageViewer from "./components/ImageViewer.svelte";
  import UserRegister from "./components/UserRegister.svelte";
  import UserLogin from "./components/UserLogin.svelte";

  registerServiceWorker();

  onMount(() => {
    if (performance.navigation.type === 1 && navigator.serviceWorker) {
      // The page was refreshed; send the service worker a
      // REFRESH message so that if the user's logged in,
      // we can keep them logged in
      navigator.serviceWorker.ready.then(async (registration) => {
        if (registration.active) {
          registration.active.postMessage({
            type: "REFRESH",
          });

          $currentUser = await getCurrentUser();
        }
      });
    }
  });

  let selectedImage: ImageData | null = null;
  let refreshTrigger = 0;

  function handleImageSelect(event: CustomEvent<ImageData>) {
    selectedImage = event.detail;
  }

  function handleImagesLoaded(event: CustomEvent<ImageData[]>) {
    if (selectedImage) {
      const imageId = selectedImage.id;
      const images = event.detail;
      selectedImage = images.find((img) => img.id === imageId);
    }
  }

  function handleImageUpdate() {
    refreshTrigger++;
  }

  function handleImageClose() {
    selectedImage = null;
  }

  function handleUploadSuccess() {
    // Trigger gallery refresh
    refreshTrigger++;
    selectedImage = null;
  }

  function handleLoginSuccess(event: CustomEvent) {
    $currentUser = event.detail;
    $currentView = "upload";
  }

  function showRegisterView() {
    $currentView = "register";
  }

  function showLoginView() {
    $currentView = "login";
  }
</script>

<div class="app">
  <Header />

  <main>
    <div class="container">
      {#if $currentView === "upload"}
        <UploadForm
          on:uploadSuccess={handleUploadSuccess}
        />
      {:else if $currentView === "gallery"}
        <ImageGallery
          on:imageSelect={handleImageSelect}
          on:imagesLoaded={handleImagesLoaded}
          refresh={refreshTrigger}
        />

        {#if selectedImage}
          <ImageViewer
            image={selectedImage}
            on:close={handleImageClose}
            on:imageUpdate={handleImageUpdate}
          />
        {/if}
      {:else if $currentView === "register"}
        <UserRegister
          on:registrationSuccess={showLoginView}
        />
      {:else if $currentView === "login"}
        <UserLogin
          on:loginSuccess={handleLoginSuccess}
          on:registerClicked={showRegisterView}
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
