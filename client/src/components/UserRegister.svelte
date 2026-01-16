<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { userRegisterUrl } from "../utils/api.ts";

  const dispatch = createEventDispatcher();

  interface Message {
    msg: string;
    type: "success" | "error";
  }

  let username: string | null = $state(null);
  let password: string | null = $state(null);
  let confirmPassword: string | null = $state(null);
  let registering: boolean = $state(false);
  let messages: Message[] = $state([]);

  async function handleSubmit(event: CustomEvent) {
    event.preventDefault();

    // Clear previous messages
    messages = [];
    registering = true;

    // Trim input values
    if (username) username = username.trim();
    if (password) password = password.trim();
    if (confirmPassword) confirmPassword = confirmPassword.trim();

    // Validate entered password
    if (!validatePassword()) {
      registering = false;
      return;
    }

    try {
      // Send user registration request
      const response = await fetch(userRegisterUrl(), {
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

        if (data.error) {
          showMessage(`Registration error: ${data.error}`, "error");
        } else {
          showMessage("Registration failed", "error");
        }
      }
    } catch (error) {
      showMessage(`Registration request failed: ${error}`, "error");
      console.error("Registration request failed:", error);
    } finally {
      registering = false;
    }
  }

  function showMessage(msg: string, type: "success" | "error") {
    messages.push({ msg, type });
  }

  function validatePassword(): boolean {
    const noConfirmMatch: boolean = password !== confirmPassword;
    const tooShort: boolean = password.length < 8;
    const noUpper: boolean = !/[A-Z]/.test(password);
    const noLower: boolean = !/[a-z]/.test(password);
    const noNumber: boolean = !/[\d]/.test(password);

    if (noConfirmMatch) {
      showMessage("Passwords do not match", "error");
    }
    if (tooShort) {
      showMessage("Password must be at least 8 characters long", "error");
    }
    if (noUpper) {
      showMessage("Password must contain at least one upper-case letter", "error");
    }
    if (noLower) {
      showMessage("Password must contain at least one lower-case letter", "error");
    }
    if (noNumber) {
      showMessage("Password must contain at least one number", "error");
    }

    return !(noConfirmMatch || tooShort || noUpper || noLower || noNumber);
  }

  function resetForm() {
    username = null;
    password = null;
    confirmPassword = null;
    messages = [];

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
          class="btn"
          type="submit"
          disabled={registering}
          >
          Register
        </button>
      </div>
    </form>
    {#if messages.length !== 0}
      {#each messages as message}
        <div class="message {message.type}">
          {message.msg}
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .register-card {
    background: none;
    padding: 20px;
  }

  small {
    color: var(--im-label);
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

  .message {
    margin-top: 12px;
    padding: 12px;
    color: var(--im-text);
    text-align: center;
    font-weight: 500;
  }

  .message.error {
    background: darkred;
  }
</style>
