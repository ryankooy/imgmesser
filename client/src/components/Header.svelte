<script lang="ts">
  import { currentView, currentUser, logOut } from "../store.ts";

  let user: string | null = $state(null);
  let userLoggedIn: boolean = $state(false);

  function showLoginView() {
    $currentView = "login";
  }

  function showGalleryView() {
    $currentView = "gallery";
  }

  async function logOutUser() {
    const userLoggedOut = await logOut();
    if (userLoggedOut) {
      $currentUser = null;
      showLoginView();
    }
  }

  $effect(() => {
    user = $currentUser;
    userLoggedIn = user != null;
    if (userLoggedIn) showGalleryView();
  });
</script>

<header>
  <div class="container">
    <div class="logo">
      <span class="title">ImgMesser</span>
    </div>
    {#if userLoggedIn}
      <div>Hi, {user}</div>
      <nav>
        <button onclick={logOutUser}>Log Out</button>
      </nav>
    {/if}
  </div>
</header>

<style>
  header {
    background: var(--im-header-gold);
    color: var(--im-text);
    padding: 20px 0;
    border-bottom: var(--im-border);
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
    color: var(--im-text);
    text-decoration: none;
    font-weight: 500;
    transition: opacity 0.2s;
  }

  nav button:hover {
    opacity: 0.8;
    cursor: pointer;
  }
</style>
