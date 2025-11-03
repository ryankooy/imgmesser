<script lang="ts">
  import Header from "./components/Header.svelte";
  import Footer from "./components/Footer.svelte";
  import UploadForm from "./components/UploadForm.svelte";
  import ImageGallery from "./components/ImageGallery.svelte";

  import { currentView } from "./store.ts";
  $currentView = "upload";

  export interface ImageData {
    key: string;
    size: number;
    last_modified: string;
    content_type: string;
  }
</script>

<div class="app">
  <Header />

  <main>
    <div class="container">
      {#if $currentView === "upload"}
        <UploadForm />
      {:else if $currentView === "gallery"}
        <ImageGallery />
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
