<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { apiUrl } from "../store.ts";

  const dispatch = createEventDispatcher();

  let username: string | null = $state(null);
  let password: string | null = $state(null);
  let confirmPassword: string | null = $state(null);
  let registering: boolean = $state(false);
  let message: string = $state("");
  let messageType: "success" | "error" | "" = $state("");

  async function handleSubmit(event: CustomEvent) {
    event.preventDefault();

    // Clear previous messages
    message = "";
    messageType = "";
    registering = true;

    if (password !== confirmPassword) {
      showMessage("Passwords do not match", "error");
    }

    try {
      // Send user registration request
      const response = await fetch(`${apiUrl}/register`, {
        method: "POST",
        body: JSON.stringify({ username, password }),
        headers: { "Content-Type": "application/json" }
      });

      if (response.ok) {
        showMessage("Registration successful", "success");

        // Reset form after successful registration
        setTimeout(() => {
          resetForm();
        }, 2000);
      } else {
        const data = await response.json();
        showMessage(`Registration error: ${data.error}`, "error");
      }
    } catch (error) {
      showMessage(`Registration request failed: ${error}`, "error");
      console.error("Registration request failed:", error);
    } finally {
      registering = false;
    }
  }

  function showMessage(msg: string, type: "success" | "error") {
    message = msg;
    messageType = type;
  }

  function resetForm() {
    username = null;
    password = null;
    confirmPassword = null;
    message = "";
    messageType = "";

    // Reset inputs
    const usernameInput = document.getElementById("username") as HTMLInputElement;
    if (usernameInput) usernameInput.value = "";
    const passwordInput = document.getElementById("password") as HTMLInputElement;
    if (passwordInput) passwordInput.value = "";
    const confirmPasswordInput = document.getElementById("confirmPassword") as HTMLInputElement;
    if (confirmPasswordInput) confirmPasswordInput.value = "";

    dispatch("registrationSuccess");
  }
</script>

<div class="register-card" id="register">
  <div class="inner">
    <h2>Registration</h2>
    <small>Create an account.</small>
    <br /><br />

    <form onsubmit={handleSubmit}>
      <div class="field">
        <label for="username">Username</label>
        <input type="text" id="username" bind:value={username} required />
      </div>

      <div class="field">
        <label for="password">Password</label>
        <input type="password" id="password" bind:value={password} required />
      </div>

      <div class="field">
        <label for="confirmPassword">Confirm Password</label>
        <input type="password" id="confirmPassword" bind:value={confirmPassword} required />
      </div>

      <div class="field">
        <button
          type="submit"
          disabled={registering}
          class="submit-btn"
        >
          Register
        </button>
      </div>

      {#if message}
        <div class="message {messageType}">
          {message}
        </div>
      {/if}
    </form>
  </div>
</div>

<style>
  .register-card {
    background: none;
    padding: 20px;
  }

  h2 {
    margin: 0;
    color: var(--im-header-gold);
    font-size: 24px;
  }

  small {
    color: #666;
  }

  .inner {
    padding: 20px;
    border: var(--im-border);
  }

  form {
    color: ghostwhite;
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
    background: none;
    border: var(--im-border);
    color: var(--im-header-gold);
    font-size: 16px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.2s;
  }

  .submit-btn:hover:not(:disabled) {
    background: var(--im-hover);
  }

  .submit-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    transform: none;
  }

  .message {
    padding: 12px;
    color: ghostwhite;
    text-align: center;
    font-weight: 500;
  }

  .message.error {
    background: darkred;
  }
</style>
