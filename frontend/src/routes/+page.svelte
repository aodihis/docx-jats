<script lang="ts">
  import { convertDocx } from '$lib/api';
  import type { ConvertResponse } from '$lib/types';

  let file = $state<File | null>(null);
  let loading = $state(false);
  let result = $state<ConvertResponse | null>(null);
  let errorMsg = $state<string | null>(null);

  function onFileChange(e: Event) {
    const input = e.target as HTMLInputElement;
    file = input.files?.[0] ?? null;
    result = null;
    errorMsg = null;
  }

  async function onSubmit(e: Event) {
    e.preventDefault();
    if (!file) return;

    loading = true;
    result = null;
    errorMsg = null;

    try {
      result = await convertDocx(file);
    } catch (err) {
      errorMsg = err instanceof Error ? err.message : 'Unknown error';
    } finally {
      loading = false;
    }
  }

  function downloadXml() {
    if (!result?.xml) return;
    const blob = new Blob([result.xml], { type: 'application/xml' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'output.jats.xml';
    a.click();
    URL.revokeObjectURL(url);
  }
</script>

<main>
  <h1>docx → JATS XML</h1>
  <p class="subtitle">Upload a Word manuscript to get simplified JATS XML.</p>

  <form onsubmit={onSubmit}>
    <div class="upload-row">
      <input
        type="file"
        accept=".docx"
        onchange={onFileChange}
        disabled={loading}
      />
      <button type="submit" disabled={!file || loading}>
        {loading ? 'Converting…' : 'Convert'}
      </button>
    </div>
  </form>

  {#if errorMsg}
    <div class="error-box">
      <strong>Error:</strong> {errorMsg}
    </div>
  {/if}

  {#if result}
    <section class="result">
      <!-- Metadata -->
      <div class="metadata">
        <h2>Document info</h2>
        <ul>
          <li><span>Title:</span> {result.metadata.title ?? '—'}</li>
          <li><span>Authors:</span> {result.metadata.author_count}</li>
          <li><span>Sections:</span> {result.metadata.section_count}</li>
          <li><span>References:</span> {result.metadata.reference_count}</li>
          <li><span>Abstract:</span> {result.metadata.has_abstract ? 'Yes' : 'No'}</li>
        </ul>
      </div>

      <!-- Warnings -->
      {#if result.warnings.length > 0}
        <details class="warnings">
          <summary>Warnings ({result.warnings.length})</summary>
          <ul>
            {#each result.warnings as w}
              <li>{w}</li>
            {/each}
          </ul>
        </details>
      {/if}

      <!-- XML preview -->
      <div class="xml-header">
        <h2>Generated XML</h2>
        <button onclick={downloadXml}>Download .jats.xml</button>
      </div>
      <pre class="xml-preview"><code>{result.xml}</code></pre>
    </section>
  {/if}
</main>

<style>
  main {
    max-width: 860px;
    margin: 2rem auto;
    padding: 0 1rem;
  }

  h1 {
    font-size: 1.75rem;
  }

  .subtitle {
    color: #555;
    margin-bottom: 1.5rem;
  }

  .upload-row {
    display: flex;
    gap: 0.75rem;
    align-items: center;
    flex-wrap: wrap;
  }

  button {
    padding: 0.45rem 1.1rem;
    border: 1px solid #333;
    border-radius: 4px;
    background: #1a1a1a;
    color: #fff;
    font-size: 0.95rem;
  }

  button:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .error-box {
    margin-top: 1rem;
    padding: 0.75rem 1rem;
    background: #fff0f0;
    border: 1px solid #f99;
    border-radius: 4px;
    color: #c00;
  }

  .result {
    margin-top: 2rem;
  }

  .metadata ul {
    list-style: none;
    padding: 0;
    margin: 0.5rem 0 1rem;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 0.3rem;
  }

  .metadata li {
    font-size: 0.9rem;
  }

  .metadata li span {
    font-weight: 600;
  }

  .warnings {
    margin-bottom: 1rem;
    padding: 0.5rem 0.75rem;
    background: #fffbea;
    border: 1px solid #e0c060;
    border-radius: 4px;
    font-size: 0.9rem;
  }

  .warnings summary {
    cursor: pointer;
    font-weight: 600;
  }

  .warnings ul {
    margin: 0.5rem 0 0;
    padding-left: 1.25rem;
  }

  .xml-header {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 0.5rem;
  }

  .xml-preview {
    background: #1e1e1e;
    color: #d4d4d4;
    padding: 1rem;
    border-radius: 4px;
    overflow: auto;
    max-height: 500px;
    font-size: 0.82rem;
    line-height: 1.5;
    white-space: pre;
  }
</style>
