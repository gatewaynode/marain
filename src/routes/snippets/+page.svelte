<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  interface Snippet {
    id: string;
    entity_type: string;
    fields: {
      title: string;
      body: string;
      status?: string;
      [key: string]: any;
    };
    created_at: string;
    updated_at: string;
  }

  let title = $state("");
  let body = $state("");
  let snippetId = $state("");
  let message = $state("");
  let snippets = $state<Snippet[]>([]);
  let currentSnippet = $state<Snippet | null>(null);

  async function createSnippet(event: Event) {
    event.preventDefault();
    try {
      const id = await invoke("create_snippet", { title, body });
      message = `Snippet created with ID: ${id}`;
      title = "";
      body = "";
      await listSnippets();
    } catch (error) {
      message = `Error: ${error}`;
    }
  }

  async function getSnippet() {
    try {
      const result = await invoke<string>("get_snippet", { id: snippetId });
      currentSnippet = JSON.parse(result) as Snippet;
      message = "Snippet loaded successfully";
    } catch (error) {
      message = `Error: ${error}`;
      currentSnippet = null;
    }
  }

  async function listSnippets() {
    try {
      const result = await invoke<string>("list_snippets");
      snippets = JSON.parse(result) as Snippet[];
      message = `Found ${snippets.length} snippets`;
    } catch (error) {
      message = `Error: ${error}`;
      snippets = [];
    }
  }

  // Load snippets on mount
  $effect(() => {
    listSnippets();
  });
</script>

<main class="container">
  <h1>Snippet Database Test</h1>
  
  <div class="section">
    <h2>Create New Snippet</h2>
    <form onsubmit={createSnippet}>
      <input 
        placeholder="Title" 
        bind:value={title} 
        required
      />
      <textarea 
        placeholder="Body content" 
        bind:value={body} 
        required
        rows="4"
      />
      <button type="submit">Create Snippet</button>
    </form>
  </div>

  <div class="section">
    <h2>Get Snippet by ID</h2>
    <div class="row">
      <input 
        placeholder="Snippet ID" 
        bind:value={snippetId}
      />
      <button onclick={getSnippet}>Get Snippet</button>
    </div>
    
    {#if currentSnippet}
      <div class="snippet-display">
        <h3>{currentSnippet.fields.title}</h3>
        <p>{currentSnippet.fields.body}</p>
        <small>ID: {currentSnippet.id}</small>
        <small>Status: {currentSnippet.fields.status || 'N/A'}</small>
      </div>
    {/if}
  </div>

  <div class="section">
    <h2>All Snippets</h2>
    <button onclick={listSnippets}>Refresh List</button>
    
    {#if snippets.length > 0}
      <div class="snippets-list">
        {#each snippets as snippet}
          <div class="snippet-item">
            <h4>{snippet.fields.title}</h4>
            <p>{snippet.fields.body}</p>
            <small>ID: {snippet.id}</small>
          </div>
        {/each}
      </div>
    {:else}
      <p>No snippets found</p>
    {/if}
  </div>

  {#if message}
    <div class="message">{message}</div>
  {/if}
</main>

<style>
  .container {
    max-width: 800px;
    margin: 0 auto;
    padding: 2rem;
  }

  .section {
    margin-bottom: 2rem;
    padding: 1rem;
    border: 1px solid #ddd;
    border-radius: 8px;
  }

  h2 {
    margin-top: 0;
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  input, textarea, button {
    padding: 0.5rem;
    border: 1px solid #ccc;
    border-radius: 4px;
    font-size: 1rem;
  }

  textarea {
    resize: vertical;
    min-height: 100px;
  }

  button {
    background-color: #007bff;
    color: white;
    cursor: pointer;
    border: none;
  }

  button:hover {
    background-color: #0056b3;
  }

  .row {
    display: flex;
    gap: 1rem;
  }

  .snippet-display, .snippet-item {
    margin-top: 1rem;
    padding: 1rem;
    background-color: #f8f9fa;
    border-radius: 4px;
  }

  .snippet-item {
    margin-bottom: 0.5rem;
  }

  .snippet-item h4 {
    margin: 0 0 0.5rem 0;
  }

  .snippets-list {
    margin-top: 1rem;
    max-height: 400px;
    overflow-y: auto;
  }

  .message {
    margin-top: 1rem;
    padding: 1rem;
    background-color: #d4edda;
    border: 1px solid #c3e6cb;
    border-radius: 4px;
    color: #155724;
  }

  small {
    display: block;
    color: #666;
    margin-top: 0.25rem;
  }
</style>