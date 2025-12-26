<script lang="ts">
  import { onMount } from "svelte";
  import { currentView, currentUser } from "./store.ts";
  import type { ImageData, ImageMeta } from "./store.ts";
  import { getCurrentUser } from "./utils/api.ts";
  import { handlePageRefresh, registerServiceWorker } from "./utils/app.ts";
  import "./styles/app.css";

  import Header from "./components/Header.svelte";
  import Footer from "./components/Footer.svelte";
  import UploadForm from "./components/UploadForm.svelte";
  import ImageGallery from "./components/ImageGallery.svelte";
  import ImageViewer from "./components/ImageViewer.svelte";
  import UserRegister from "./components/UserRegister.svelte";
  import UserLogin from "./components/UserLogin.svelte";

  registerServiceWorker();

  onMount(() => {
    handlePageRefresh();
    (async () => {
      $currentUser = await getCurrentUser();
      if ($currentUser == null) setLoginView();
    })();
  });

  let selectedImage: ImageData | null = $state(null);
  let selectedImageId: string | null = $state(null);
  let showUploadModal: boolean = $state(false);
  let selectingNext: boolean = $state(false);
  let selectingPrev: boolean = $state(false);
  let imageIds: string[] = $state([]);

  // Triggers for reloading gallery
  let refreshAllTrigger = $state(0);
  let refreshOneTrigger = $state(0);

  function handleImageSelect(event: CustomEvent<ImageData>) {
    selectingNext = selectingPrev = false;
    selectedImage = event.detail;
    selectedImageId = selectedImage.id;
  }

  function handleImagesLoaded(event: CustomEvent<ImageMeta[]>) {
    const images = event.detail;
    imageIds = images.map((img) => img.id);

    if (selectedImage) {
      const imageId: string = selectedImageId;
      selectedImage.meta = images.find((img) => img.id === imageId);
    }
  }

  function handleImageUpdate(event: Event) {
    const updateOne = event.detail;
    if (updateOne) {
      refreshOneTrigger++;
    } else {
      handleImageClose();
      refreshAllTrigger++;
    }
  }

  function handleImageClose() {
    selectedImage = null;
    selectedImageId = null;
  }

  function handleSelectNextImage() {
    if (selectedImageId) {
      const index: number = imageIds.indexOf(selectedImageId);
      if (index !== -1 && index !== imageIds.length - 1) {
        selectingNext = true;
        refreshOneTrigger++;
      }
    }
  }

  function handleSelectPrevImage() {
    if (selectedImageId) {
      const index: number = imageIds.indexOf(selectedImageId);
      if (index > 0) {
        selectingPrev = true;
        refreshOneTrigger++;
      }
    }
  }

  function handleUploadModalOpen() {
    showUploadModal = true;
  }

  function handleUploadModalClose() {
    showUploadModal = false;
  }

  function handleUploadSuccess() {
    refreshAllTrigger++;
    handleImageClose();
  }

  function handleLoginSuccess(event: Event) {
    $currentUser = event.detail;
    $currentView = "gallery";
  }

  function setRegisterView() {
    $currentView = "register";
  }

  function setLoginView() {
    $currentView = "login";
  }

  function handleSelectDataUrl(event: Event) {
    if (selectedImage) selectedImage.url = event.detail;
  }
</script>

<div class="app">
  <Header />

  <main>
    <div class="container">
      {#if $currentView === "gallery"}
        <ImageGallery
          on:imageSelect={handleImageSelect}
          on:imagesLoaded={handleImagesLoaded}
          on:upload={handleUploadModalOpen}
          selectedId={selectedImageId}
          selectingNext={selectingNext}
          selectingPrev={selectingPrev}
          refreshAll={refreshAllTrigger}
          refreshOne={refreshOneTrigger}
        />

        {#if selectedImage}
          <ImageViewer
            image={selectedImage}
            imageIds={imageIds}
            on:close={handleImageClose}
            on:imageUpdate={handleImageUpdate}
            on:selectNextImage={handleSelectNextImage}
            on:selectPrevImage={handleSelectPrevImage}
            on:selectDataUrl={handleSelectDataUrl}
          />
        {:else if showUploadModal}
          <UploadForm
            on:uploadSuccess={handleUploadSuccess}
            on:close={handleUploadModalClose}
          />
        {/if}
      {:else if $currentView === "register"}
        <UserRegister
          on:registrationSuccess={setLoginView}
        />
      {:else if $currentView === "login"}
        <UserLogin
          on:loginSuccess={handleLoginSuccess}
          on:registerClicked={setRegisterView}
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
    font-family: century-gothic, sans-serif;
    background: black;
  }

  :global(button) {
    font-family: inherit;
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
