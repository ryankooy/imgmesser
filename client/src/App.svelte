<script lang="ts">
  import { onMount } from "svelte";
  import { currentView, currentUser, galleryPageCache } from "./store.ts";
  import type { GalleryPagination, ImageData, ImageMeta } from "./store.ts";
  import { getCurrentUser } from "./utils/api.ts";
  import { handlePageRefresh, registerServiceWorker } from "./utils/app.ts";
  import "./styles/app.css";

  import Header from "./components/Header.svelte";
  import Footer from "./components/Footer.svelte";
  import UploadForm from "./components/UploadForm.svelte";
  import ImageGallery from "./components/ImageGallery.svelte";
  import ImageEditor from "./components/ImageEditor.svelte";
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
  let imageIds: string[] = $state([]);
  let pagination: GalleryPagination | null = $state(null);
  let imageDataUrls: Map<string, string> = $state(new Map());
  let imageVersions: Map<string, string> = $state(new Map());

  // Triggers for reloading gallery
  let nextImageTrigger: number = $state(0);
  let prevImageTrigger: number = $state(0);
  let nextPageTrigger: number = $state(0);
  let prevPageTrigger: number = $state(0);
  let refreshAllTrigger: number = $state(0);
  let refreshOneTrigger: number = $state(0);

  function handleImageSelect(event: CustomEvent<ImageData>) {
    selectedImage = event.detail;
    selectedImageId = selectedImage.id;
  }

  function handleImagesLoaded(event: CustomEvent<ImageMeta[]>) {
    const images = event.detail;
    imageIds = images.map((img) => img.id);

    if (selectedImage) {
      selectedImage.meta = images.find((img) => img.id === selectedImageId);
    }
  }

  function handleImageUpdate(event: Event) {
    const state = event.detail;

    if (state === "deleting") {
      closeSelectedImage();
      $galleryPageCache.clear();
      refreshAllTrigger++;
    } else {
      refreshOneTrigger++;
    }
  }

  function closeSelectedImage() {
    selectedImage = null;
    selectedImageId = null;
  }

  function handleSelectNextImage() {
    if (selectedImageId) {
      const index: number = imageIds.indexOf(selectedImageId);

      if (index !== -1 && index !== imageIds.length - 1) {
        nextImageTrigger++;
      } else if (pagination && pagination.has_more) {
        closeSelectedImage();
        nextPageTrigger++;
      }
    }
  }

  function handleSelectPrevImage() {
    if (selectedImageId) {
      const index: number = imageIds.indexOf(selectedImageId);

      if (index > 0) {
        prevImageTrigger++;
      } else if (pagination && pagination.current_page > 1) {
        closeSelectedImage();
        prevPageTrigger++;
      }
    }
  }

  function handleSelectDataUrl(event: Event) {
    if (selectedImage) selectedImage.url = event.detail;
  }

  function handlePaginationUpdated(event: CustomEvent<GalleryPagination>) {
    pagination = event.detail;
  }

  function handleUploadModalOpen() {
    showUploadModal = true;
  }

  function handleUploadModalClose() {
    showUploadModal = false;
  }

  function handleUploadSuccess() {
    closeSelectedImage();
    $galleryPageCache.clear();
    refreshAllTrigger++;
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
</script>

<div class="app">
  <Header />

  <main>
    <div class="container">
      {#if $currentView === "gallery"}
        <ImageGallery
          imageDataUrls={imageDataUrls}
          imageVersions={imageVersions}
          nextImageTrigger={nextImageTrigger}
          nextPageTrigger={nextPageTrigger}
          prevImageTrigger={prevImageTrigger}
          prevPageTrigger={prevPageTrigger}
          refreshAll={refreshAllTrigger}
          refreshOne={refreshOneTrigger}
          selectedId={selectedImageId}
          on:imageSelect={handleImageSelect}
          on:imagesLoaded={handleImagesLoaded}
          on:paginationUpdated={handlePaginationUpdated}
          on:upload={handleUploadModalOpen}
        />

        {#if selectedImage}
          <ImageEditor
            image={selectedImage}
            imageIds={imageIds}
            pagination={pagination}
            on:close={closeSelectedImage}
            on:imageUpdate={handleImageUpdate}
            on:selectDataUrl={handleSelectDataUrl}
            on:selectNextImage={handleSelectNextImage}
            on:selectPrevImage={handleSelectPrevImage}
          />
        {:else if showUploadModal}
          <UploadForm
            on:close={handleUploadModalClose}
            on:uploadSuccess={handleUploadSuccess}
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
