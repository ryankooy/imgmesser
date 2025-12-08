<script lang="ts">
  const currentYear = new Date().getFullYear();
</script>

<footer>
  <div class="container">
    <p>&copy; {currentYear} ImgMesser</p>
    <div class="links">
      <a href="https://github.com/ryankooy/imgmesser" target="_blank" rel="noopener">GitHub</a>
    </div>
  </div>
</footer>

<style>
  footer {
    background: none;
    color: ghostwhite;
    padding: 32px 0;
    margin-top: auto;
    border-top: var(--im-border);
  }

  .container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 0 20px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 16px;
  }

  p {
    margin: 0;
    color: #cbd5e0;
  }

  .links {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .links a {
    color: #e2e8f0;
    text-decoration: none;
    transition: color 0.2s;
  }

  .links a:hover {
    color: #667eea;
  }

  @media (max-width: 640px) {
    .container {
      flex-direction: column;
      text-align: center;
    }
  }
</style>
