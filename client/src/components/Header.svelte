<script lang="ts">
  import { currentView, userLoggedIn } from "../store.ts";

  function showUploadView() {
    $currentView = "upload";
  }

  function showGalleryView() {
    $currentView = "gallery";
  }

  function showLoginView() {
    $currentView = "login";
  }
</script>

<header>
  <div class="container">
    <div class="logo">
      <span class="title">ImgMesser</span>
    </div>
    <nav>
      {#if $userLoggedIn}
        <button on:click={showUploadView}>Upload</button>
        <button on:click={showGalleryView}>Gallery</button>
        <button>Log Out</button>
      {:else}
        <!-- TODO: maybe remove login nav button -->
        <button on:click={showLoginView}>Log In</button>
      {/if}
    </nav>
  </div>
</header>

<style>
  header {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    padding: 20px 0;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }

  .container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 0 20px;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .logo {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 24px;
    font-weight: 700;
  }

  nav {
    display: flex;
    gap: 24px;
  }

  nav button {
    padding: 0;
    border: none;
    background: none;
    color: white;
    text-decoration: none;
    font-weight: 500;
    transition: opacity 0.2s;
  }

  nav button:hover {
    opacity: 0.8;
    cursor: pointer;
  }
</style>
