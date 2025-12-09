<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { apiUrl } from "../store.ts";

  const dispatch = createEventDispatcher();

  let username: string | null = $state(null);
  let password: string | null = $state(null);
  let loggingIn: boolean = $state(false);
  let message: string = $state("");
  let messageType: "success" | "error" | "" = $state("");

  async function handleLogin() {
    event.preventDefault();

    // Clear previous messages
    message = "";
    messageType = "";
    loggingIn = true;

    try {
      // Send user login request
      const response = await fetch(`${apiUrl}/login`, {
        method: "POST",
        body: JSON.stringify({ username, password }),
        headers: { "Content-Type": "application/json" }
      });

      const data = await response.json();

      if (response.ok) {
        dispatch("loginSuccess", data.user.username);
        showMessage("Login successful", "success");

        // Reset form after successful login
        resetForm();
      } else if (data.error) {
        showMessage(`Login error: ${data.error}`, "error");
      } else {
        showMessage("User not found", "error");
      }
    } catch (error) {
      showMessage(`Login request failed: ${error}`, "error");
      console.error("Login request failed:", error);
    } finally {
      loggingIn = false;
    }
  }

  function showMessage(msg: string, type: "success" | "error") {
    message = msg;
    messageType = type;
  }

  function resetForm() {
    username = null;
    password = null;
    message = "";
    messageType = "";

    // Reset inputs
    const usernameInput = document.getElementById("username") as HTMLInputElement;
    if (usernameInput) usernameInput.value = "";
    const passwordInput = document.getElementById("password") as HTMLInputElement;
    if (passwordInput) passwordInput.value = "";
  }

  function handleRegisterClicked() {
    dispatch("registerClicked");
  }
</script>

<div class="login-card" id="login">
  <div class="inner">
    <h2>User Login</h2>

    <form onsubmit={handleLogin}>
      <div class="field">
        <label for="username">Username</label>
        <input type="text" id="username" bind:value={username} required />
      </div>

      <div class="field">
        <label for="password">Password</label>
        <input type="password" id="password" bind:value={password} required />
      </div>

      <div class="field">
        <button
          type="submit"
          disabled={loggingIn}
          class="submit-btn"
        >
          Log In
        </button>
      </div>

      {#if message}
        <div class="message {messageType}">
          {message}
        </div>
      {/if}
    </form>

    <p>
      Need to
      <span class="clickable"
        onclick={handleRegisterClicked}
        role="button"
        tabindex="0"
      >
        register
      </span>
      ?
    </p>
  </div>
</div>

<style>
  .login-card {
    background: none;
    padding: 20px;
  }

  h2 {
    margin: 0 0 24px 0;
    color: var(--im-header-gold);
    font-size: 24px;
  }

  .inner {
    padding: 20px;
    border: var(--im-border);
  }

  form {
    color: var(--im-text);
    display: flex;
    flex-direction: column;
    align-content: flex-start;
    flex-wrap: wrap;
    row-gap: 20px;
  }

  .field {
    display: flex;
    justify-content: flex-end;
  }

  .field label {
    padding: .5em 1em .5em 0;
    flex: 1;
  }

  .field input {
    flex: 2;
    padding: 8px;
    border: 1px solid #667eea;
    background: #f8f9ff;
    transition: border-color 0.2s;
  }

  input[type="text"]:hover:not(:disabled) {
    border-color: #764ba2;
  }

  .submit-btn {
    padding: 14px 24px;
    background: none;
    border: var(--im-border);
    color: var(--im-text);
    font-size: 16px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.2s;
  }

  .submit-btn:hover:not(:disabled) {
    background: var(--im-hover-gold);
  }

  .submit-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    transform: none;
  }

  p {
    color: var(--im-text);
  }

  .clickable {
    color: blue;
    text-decoration: underline;
    cursor: pointer;
  }

  .message {
    padding: 12px;
    color: var(--im-text);
    text-align: center;
    font-weight: 500;
  }

  .message.error {
    background: darkred;
  }
</style>
