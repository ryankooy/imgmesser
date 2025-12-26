<script lang="ts">
  import { currentUser, currentView } from "../store.ts";
  import { logOut } from "../utils/api.ts";

  let user: string | null = $state(null);

  async function logOutUser() {
    const userLoggedOut = await logOut();
    if (userLoggedOut) {
      $currentUser = null;
      $currentView = "login";
    }
  }

  $effect(() => {
    user = $currentUser;
    if (user != null) $currentView = "gallery";
  });
</script>

<header>
  <div class="container">
    <div class="logo">
      <span class="title">ImgMesser</span>
    </div>
    {#if user != null}
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
