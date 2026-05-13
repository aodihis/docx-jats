<script lang="ts">
  import { convertDocx } from '$lib/api';
  import type { ConvertResponse } from '$lib/types';

  let file = $state<File | null>(null);
  let loading = $state(false);
  let result = $state<ConvertResponse | null>(null);
  let errorMsg = $state<string | null>(null);
  let dragging = $state(false);
  let copied = $state(false);

  let fileInput: HTMLInputElement;

  function pickFile(picked: File | null | undefined) {
    if (!picked) return;
    if (!picked.name.endsWith('.docx')) {
      errorMsg = 'Only .docx files are supported.';
      return;
    }
    file = picked;
    result = null;
    errorMsg = null;
  }

  function onFileChange(e: Event) {
    const input = e.target as HTMLInputElement;
    pickFile(input.files?.[0]);
  }

  function onDragOver(e: DragEvent) {
    e.preventDefault();
    dragging = true;
  }

  function onDragLeave() {
    dragging = false;
  }

  function onDrop(e: DragEvent) {
    e.preventDefault();
    dragging = false;
    pickFile(e.dataTransfer?.files?.[0]);
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

  async function copyXml() {
    if (!result?.xml) return;
    await navigator.clipboard.writeText(result.xml);
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }

  function clearFile() {
    file = null;
    result = null;
    errorMsg = null;
    if (fileInput) fileInput.value = '';
  }
</script>

<div class="min-h-screen bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900 text-slate-100">
  <!-- Header -->
  <header class="border-b border-slate-700/50 bg-slate-900/60 backdrop-blur-sm sticky top-0 z-10">
    <div class="max-w-4xl mx-auto px-6 py-4 flex items-center gap-3">
      <div class="w-8 h-8 rounded-lg bg-indigo-500 flex items-center justify-center text-white font-bold text-sm">J</div>
      <span class="font-semibold text-lg tracking-tight">docx-jats</span>
      <span class="ml-auto text-xs text-slate-400 bg-slate-800 px-2.5 py-1 rounded-full border border-slate-700">JATS XML converter</span>
    </div>
  </header>

  <main class="max-w-4xl mx-auto px-6 py-12">
    <!-- Hero -->
    <div class="text-center mb-10">
      <h1 class="text-4xl font-bold tracking-tight mb-3">
        Convert <span class="text-indigo-400">.docx</span> to JATS XML
      </h1>
      <p class="text-slate-400 text-lg">Drop a Word manuscript below and get structured JATS XML instantly.</p>
    </div>

    <!-- Upload card -->
    <form onsubmit={onSubmit}>
      <div
        role="button"
        tabindex="0"
        class="relative rounded-2xl border-2 border-dashed transition-all duration-200 cursor-pointer
               {dragging
                 ? 'border-indigo-400 bg-indigo-500/10 scale-[1.01]'
                 : file
                   ? 'border-emerald-500/60 bg-emerald-500/5'
                   : 'border-slate-600 bg-slate-800/40 hover:border-slate-500 hover:bg-slate-800/60'}"
        ondragover={onDragOver}
        ondragleave={onDragLeave}
        ondrop={onDrop}
        onclick={() => !file && fileInput.click()}
        onkeydown={(e) => e.key === 'Enter' && !file && fileInput.click()}
      >
        <input
          bind:this={fileInput}
          type="file"
          accept=".docx"
          class="sr-only"
          onchange={onFileChange}
          disabled={loading}
        />

        <div class="flex flex-col items-center justify-center py-16 px-6 text-center">
          {#if file}
            <!-- File selected state -->
            <div class="w-16 h-16 rounded-full bg-emerald-500/20 flex items-center justify-center mb-4">
              <svg class="w-8 h-8 text-emerald-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                <path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
            <p class="font-semibold text-emerald-400 text-lg mb-1">{file.name}</p>
            <p class="text-slate-400 text-sm mb-5">{(file.size / 1024).toFixed(1)} KB · .docx</p>
            <div class="flex gap-3">
              <button
                type="submit"
                disabled={loading}
                class="px-6 py-2.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed
                       font-semibold text-sm transition-colors duration-150 flex items-center gap-2"
              >
                {#if loading}
                  <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
                  </svg>
                  Converting…
                {:else}
                  <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M13 10V3L4 14h7v7l9-11h-7z" />
                  </svg>
                  Convert to JATS XML
                {/if}
              </button>
              <button
                type="button"
                onclick={(e) => { e.stopPropagation(); clearFile(); }}
                class="px-4 py-2.5 rounded-xl border border-slate-600 hover:border-slate-400 text-slate-400 hover:text-slate-200
                       text-sm transition-colors duration-150"
              >
                Remove
              </button>
            </div>
          {:else}
            <!-- Empty / drag state -->
            <div class="w-16 h-16 rounded-full bg-slate-700/60 flex items-center justify-center mb-4
                        {dragging ? 'bg-indigo-500/20' : ''}">
              <svg class="w-8 h-8 {dragging ? 'text-indigo-400' : 'text-slate-400'}" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                <path stroke-linecap="round" stroke-linejoin="round" d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5m-13.5-9L12 3m0 0l4.5 4.5M12 3v13.5" />
              </svg>
            </div>
            <p class="font-semibold text-slate-200 text-lg mb-1">
              {dragging ? 'Drop it here!' : 'Drop your .docx file here'}
            </p>
            <p class="text-slate-400 text-sm mb-5">or click to browse</p>
            <span class="px-4 py-2 rounded-lg bg-slate-700 text-slate-300 text-sm border border-slate-600">
              Browse files
            </span>
          {/if}
        </div>
      </div>
    </form>

    <!-- Error -->
    {#if errorMsg}
      <div class="mt-4 flex items-start gap-3 rounded-xl bg-red-500/10 border border-red-500/30 px-4 py-3.5 text-red-300">
        <svg class="w-5 h-5 mt-0.5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126zM12 15.75h.007v.008H12v-.008z" />
        </svg>
        <div>
          <p class="font-semibold text-sm">Conversion failed</p>
          <p class="text-sm opacity-80 mt-0.5">{errorMsg}</p>
        </div>
      </div>
    {/if}

    <!-- Results -->
    {#if result}
      <div class="mt-8 space-y-6">
        <!-- Metadata cards -->
        <div>
          <h2 class="text-xs font-semibold uppercase tracking-widest text-slate-400 mb-3">Document info</h2>
          <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-3">
            {#each [
              { label: 'Title', value: result.metadata.title ?? '—' },
              { label: 'Authors', value: String(result.metadata.author_count) },
              { label: 'Sections', value: String(result.metadata.section_count) },
              { label: 'References', value: String(result.metadata.reference_count) },
              { label: 'Abstract', value: result.metadata.has_abstract ? 'Yes' : 'No' },
            ] as card}
              <div class="rounded-xl bg-slate-800/60 border border-slate-700/50 px-4 py-3">
                <p class="text-xs text-slate-400 mb-1">{card.label}</p>
                <p class="font-semibold text-slate-100 truncate" title={card.value}>{card.value}</p>
              </div>
            {/each}
          </div>
        </div>

        <!-- Warnings -->
        {#if result.warnings.length > 0}
          <details class="rounded-xl bg-amber-500/10 border border-amber-500/30 px-4 py-3 group">
            <summary class="cursor-pointer flex items-center gap-2 text-amber-300 font-semibold text-sm select-none list-none">
              <svg class="w-4 h-4 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126zM12 15.75h.007v.008H12v-.008z" />
              </svg>
              {result.warnings.length} warning{result.warnings.length > 1 ? 's' : ''}
              <svg class="w-3 h-3 ml-auto transition-transform group-open:rotate-180" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
                <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 8.25l-7.5 7.5-7.5-7.5" />
              </svg>
            </summary>
            <ul class="mt-3 space-y-1 text-amber-200/80 text-sm">
              {#each result.warnings as w}
                <li class="flex gap-2"><span class="text-amber-500 mt-0.5">·</span>{w}</li>
              {/each}
            </ul>
          </details>
        {/if}

        <!-- XML preview -->
        <div>
          <div class="flex items-center justify-between mb-3">
            <h2 class="text-xs font-semibold uppercase tracking-widest text-slate-400">Generated XML</h2>
            <div class="flex gap-2">
              <button
                onclick={copyXml}
                class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium border border-slate-600
                       hover:border-slate-400 text-slate-300 hover:text-white transition-colors duration-150"
              >
                {#if copied}
                  <svg class="w-3.5 h-3.5 text-emerald-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M4.5 12.75l6 6 9-13.5" />
                  </svg>
                  Copied!
                {:else}
                  <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M15.666 3.888A2.25 2.25 0 0013.5 2.25h-3c-1.03 0-1.9.693-2.166 1.638m7.332 0c.055.194.084.4.084.612v0a.75.75 0 01-.75.75H9a.75.75 0 01-.75-.75v0c0-.212.03-.418.084-.612m7.332 0c.646.049 1.288.11 1.927.184 1.1.128 1.907 1.077 1.907 2.185V19.5a2.25 2.25 0 01-2.25 2.25H6.75A2.25 2.25 0 014.5 19.5V6.257c0-1.108.806-2.057 1.907-2.185a48.208 48.208 0 011.927-.184" />
                  </svg>
                  Copy
                {/if}
              </button>
              <button
                onclick={downloadXml}
                class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium bg-indigo-600
                       hover:bg-indigo-500 text-white transition-colors duration-150"
              >
                <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5M16.5 12L12 16.5m0 0L7.5 12m4.5 4.5V3" />
                </svg>
                Download .jats.xml
              </button>
            </div>
          </div>
          <div class="rounded-xl bg-slate-950 border border-slate-700/50 overflow-hidden">
            <div class="flex items-center gap-1.5 px-4 py-2.5 border-b border-slate-700/50 bg-slate-900/50">
              <span class="w-3 h-3 rounded-full bg-red-500/60"></span>
              <span class="w-3 h-3 rounded-full bg-amber-500/60"></span>
              <span class="w-3 h-3 rounded-full bg-emerald-500/60"></span>
              <span class="ml-3 text-xs text-slate-500 font-mono">output.jats.xml</span>
            </div>
            <pre class="overflow-auto max-h-[480px] p-5 text-xs leading-relaxed text-slate-300 font-mono"><code>{result.xml}</code></pre>
          </div>
        </div>
      </div>
    {/if}
  </main>

  <footer class="border-t border-slate-700/40 mt-16">
    <div class="max-w-4xl mx-auto px-6 py-6 text-center text-xs text-slate-500">
      docx-jats · DOCX to JATS XML converter
    </div>
  </footer>
</div>
