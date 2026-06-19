<script lang="ts">
  import { onMount } from "svelte";
  import { currentView, currentUser, galleryPageCache, ImageStatus } from "./store.ts";
  import type { GalleryPagination, ImageData, ImageMeta } from "./store.ts";
  import { getCurrentUser } from "./utils/api.ts";
  import { handlePageRefresh, registerServiceWorker } from "./utils/app.ts";
  import "./styles/app.css";

  import Header from "./components/Header.svelte";
  import Footer from "./components/Footer.svelte";
  import UploadForm from "./components/UploadForm.svelte";
  import Gallery from "./components/Gallery.svelte";
  import Image from "./components/Image.svelte";
  import UserRegister from "./components/UserRegister.svelte";
  import UserLogin from "./components/UserLogin.svelte";

  // Register the service worker, which is responsible for intercepting
  // requests in order to authenticate users
  registerServiceWorker();

  onMount(() => {
    handlePageRefresh();

    (async () => {
      // Set the current user
      $currentUser = await getCurrentUser();

      // If no username is found, show the login page
      if ($currentUser == null) setLoginView();
    })();
  });

  let selectedImage: ImageData | null = $state(null);
  let selectedImageId: string | null = $state(null);

  let imageIds: string[] = $state([]);
  let imageDataUrls: Map<string, string> = $state(new Map());
  let imageVersions: Map<string, string> = $state(new Map());

  let pagination: GalleryPagination | null = $state(null);

  let showUploadModal: boolean = $state(false);

  // Triggers for reloading gallery
  let nextImageTrigger: number = $state(0);
  let prevImageTrigger: number = $state(0);
  let nextPageTrigger: number = $state(0);
  let prevPageTrigger: number = $state(0);
  let refreshAllTrigger: number = $state(0);
  let refreshOneTrigger: number = $state(0);

  function handleSelectImage(image: ImageData) {
    selectedImage = image;
    selectedImageId = selectedImage.id;
  }

  function handleImagesLoaded(images: ImageMeta[]) {
    // Create array of image IDs
    imageIds = images.map((img) => img.id);

    if (selectedImage) {
      // Set the selected image's metadata
      selectedImage.meta = images.find((img) => img.id === selectedImageId);
    }
  }

  function handleRefreshImage(status: ImageStatus) {
    if (status === ImageStatus.Deleting)
      handleRefreshGallery();
    else
      refreshOneTrigger++;
  }

  function handleCloseImage() {
    selectedImage = null;
    selectedImageId = null;
  }

  function handleSelectNextImage() {
    if (!selectedImageId) return;

    // Get the selected image's array index
    const index: number = imageIds.indexOf(selectedImageId);

    if (index !== -1 && index !== imageIds.length - 1) {
      // Trigger navigation to next image
      nextImageTrigger++;
    } else if (pagination && pagination.has_more) {
      // Open the first image on the next gallery page
      handleCloseImage();
      nextPageTrigger++;
    }
  }

  function handleSelectPrevImage() {
    if (!selectedImageId) return;

    // Get the selected image's array index
    const index: number = imageIds.indexOf(selectedImageId);

    if (index > 0) {
      // Trigger navigation to previous image
      prevImageTrigger++;
    } else if (pagination && pagination.current_page > 1) {
      // Open the last image on the previous gallery page
      handleCloseImage();
      prevPageTrigger++;
    }
  }

  function handleSetImageDataUrl(dataUrl: string) {
    if (selectedImage) selectedImage.url = dataUrl;
  }

  function handlePaginationUpdated(galleryPagination: GalleryPagination) {
    pagination = galleryPagination;
  }

  function handleOpenUploadModal() {
    showUploadModal = true;
  }

  function handleCloseUploadModal() {
    showUploadModal = false;
  }

  function handleRefreshGallery() {
    handleCloseImage();
    $galleryPageCache.clear();
    refreshAllTrigger++;
  }

  function handleLogIn(username: string) {
    $currentUser = username;
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
        <Gallery
          imageDataUrls={imageDataUrls}
          imageVersions={imageVersions}
          nextImageTrigger={nextImageTrigger}
          nextPageTrigger={nextPageTrigger}
          prevImageTrigger={prevImageTrigger}
          prevPageTrigger={prevPageTrigger}
          refreshAll={refreshAllTrigger}
          refreshOne={refreshOneTrigger}
          selectedId={selectedImageId}
          imagesLoaded={handleImagesLoaded}
          openUploadModal={handleOpenUploadModal}
          paginationUpdated={handlePaginationUpdated}
          selectImage={handleSelectImage}
        />

        {#if selectedImage}
          <Image
            image={selectedImage}
            imageIds={imageIds}
            pagination={pagination}
            closeImage={handleCloseImage}
            refreshImage={handleRefreshImage}
            selectNextImage={handleSelectNextImage}
            selectPrevImage={handleSelectPrevImage}
            setImageDataUrl={handleSetImageDataUrl}
          />
        {:else if showUploadModal}
          <UploadForm
            closeModal={handleCloseUploadModal}
            refreshGallery={handleRefreshGallery}
          />
        {/if}
      {:else if $currentView === "register"}
        <UserRegister setLoginView={setLoginView} />
      {:else if $currentView === "login"}
        <UserLogin
          logIn={handleLogIn}
          setRegisterView={setRegisterView}
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
