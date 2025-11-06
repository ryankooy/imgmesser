<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { API_URL } from "../store.ts";

  const dispatch = createEventDispatcher();

  let username: string | null = null;
  let password: string | null = null;
  let loggingIn = false;
  let message = "";
  let messageType: "success" | "error" | "" = "";

  async function handleLogin() {
    // Clear previous messages
    message = "";
    messageType = "";
    loggingIn = true;

    try {
      // Send user login request
      const response = await fetch(`${API_URL}/login`, {
        method: "POST",
        body: JSON.stringify({ username, password }),
        headers: { "Content-Type": "application/json" }
      });

      if (response.ok) {
        showMessage("Login successful", "success");

        // Reset form after successful login
        setTimeout(() => {
          resetForm();
        }, 2000);
      } else {
        showMessage(`❌ Login failed`, "error");
      }
    } catch (error) {
      showMessage(`❌ Login failed: ${error}`, "error");
      console.error("Login error:", error);
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

    dispatch("loginSuccess");
  }

  function showRegisterView() {
    dispatch("showRegisterView");
  }
</script>

<div class="login-card" id="login">
  <h2>Login</h2>

  <form on:submit|preventDefault={handleLogin}>
    <div class="field">
      <label for="username">Username</label>
      <input type="text" id="username" bind:value={username} required />
    </div>

    <div class="field">
      <label for="password">Password</label>
      <input type="password" id="password" bind:value={password} required />
    </div>

    <div class="field">
      <p>
        Need to
        <span class="clickable"
          on:click={showRegisterView}
          role="button"
          tabindex="0"
        >
          register
        </span>
        ?
      </p>
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
</div>

<style>
  .login-card {
    background: white;
    border-radius: 16px;
    padding: 32px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }

  h2 {
    margin: 0 0 24px 0;
    color: #333;
    font-size: 24px;
  }

  form {
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
    border-radius: 8px;
    background: #f8f9ff;
    transition: border-color 0.2s;
  }

  input[type="text"]:hover:not(:disabled) {
    border-color: #764ba2;
  }

  .submit-btn {
    padding: 14px 24px;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    border: none;
    border-radius: 8px;
    font-size: 16px;
    font-weight: 600;
    cursor: pointer;
    transition: transform 0.2s, box-shadow 0.2s;
  }

  .submit-btn:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: 0 8px 16px rgba(102, 126, 234, 0.4);
  }

  .submit-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    transform: none;
  }

  .clickable {
    color: blue;
    text-decoration: underline;
    cursor: pointer;
  }

  .message {
    padding: 12px;
    border-radius: 8px;
    text-align: center;
    font-weight: 500;
  }

  .message.success {
    background: #d4edda;
    color: #155724;
    border: 1px solid #c3e6cb;
  }

  .message.error {
    background: #f8d7da;
    color: #721c24;
    border: 1px solid #f5c6cb;
  }
</style>
