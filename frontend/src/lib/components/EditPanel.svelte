<script lang="ts">
  import { regenerateXml } from '$lib/api';
  import type { ConvertResponse, DocumentContent } from '$lib/types';

  let { result, onback }: { result: ConvertResponse; onback: () => void } = $props();

  // Deep clone so edits don't mutate the original result
  let editableDoc = $state<DocumentContent>(JSON.parse(JSON.stringify(result.document)));
  let currentXml = $state(result.xml);
  let currentWarnings = $state<string[]>(result.warnings);
  let regenerating = $state(false);
  let copied = $state(false);
  let regenError = $state<string | null>(null);
  let xmlModalOpen = $state(false);

  async function applyAndPreview() {
    regenerating = true;
    regenError = null;
    try {
      const res = await regenerateXml(editableDoc);
      currentXml = res.xml;
      currentWarnings = res.warnings;
      xmlModalOpen = true;
    } catch (err) {
      regenError = err instanceof Error ? err.message : 'Unknown error';
    } finally {
      regenerating = false;
    }
  }

  function downloadXml() {
    const blob = new Blob([currentXml], { type: 'application/xml' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'output.jats.xml';
    a.click();
    URL.revokeObjectURL(url);
  }

  async function copyXml() {
    await navigator.clipboard.writeText(currentXml);
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }

  function addAuthor() {
    editableDoc.authors = [...editableDoc.authors, { name: '' }];
  }

  function removeAuthor(i: number) {
    editableDoc.authors = editableDoc.authors.filter((_, idx) => idx !== i);
  }

  function addSection() {
    editableDoc.sections = [...editableDoc.sections, { heading: '', level: 1, body: [''] }];
  }

  function removeSection(i: number) {
    editableDoc.sections = editableDoc.sections.filter((_, idx) => idx !== i);
  }

  function addParagraph(si: number) {
    editableDoc.sections = editableDoc.sections.map((s, idx) =>
      idx === si ? { ...s, body: [...s.body, ''] } : s
    );
  }

  function removeParagraph(si: number, pi: number) {
    editableDoc.sections = editableDoc.sections.map((s, idx) =>
      idx === si ? { ...s, body: s.body.filter((_, i) => i !== pi) } : s
    );
  }

  function removeReference(i: number) {
    editableDoc.references = editableDoc.references.filter((_, idx) => idx !== i);
  }

  function onModalKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') xmlModalOpen = false;
  }
</script>

<!-- XML Preview Modal -->
{#if xmlModalOpen}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    role="dialog"
    aria-modal="true"
    aria-label="XML Preview"
    class="fixed inset-0 z-50 flex items-center justify-center p-4"
    onkeydown={onModalKeydown}
  >
    <!-- Backdrop -->
    <button
      class="absolute inset-0 bg-black/70 backdrop-blur-sm cursor-default"
      onclick={() => (xmlModalOpen = false)}
      aria-label="Close XML preview"
      tabindex="-1"
    ></button>

    <!-- Dialog panel -->
    <div class="relative z-10 w-full max-w-4xl max-h-[90vh] flex flex-col rounded-2xl bg-slate-950 border border-slate-700/50 shadow-2xl">
      <!-- Modal header -->
      <div class="flex items-center gap-1.5 px-4 py-3 border-b border-slate-700/50 bg-slate-900/50 rounded-t-2xl shrink-0">
        <span class="w-3 h-3 rounded-full bg-red-500/60"></span>
        <span class="w-3 h-3 rounded-full bg-amber-500/60"></span>
        <span class="w-3 h-3 rounded-full bg-emerald-500/60"></span>
        <span class="ml-3 text-xs text-slate-500 font-mono flex-1">output.jats.xml</span>
        <div class="flex gap-2">
          <button
            onclick={copyXml}
            class="flex items-center gap-1.5 px-3 py-1 rounded-lg text-xs font-medium border border-slate-700
                   hover:border-slate-500 text-slate-400 hover:text-white transition-colors"
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
            class="flex items-center gap-1.5 px-3 py-1 rounded-lg text-xs font-medium bg-indigo-600
                   hover:bg-indigo-500 text-white transition-colors"
          >
            <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5M16.5 12L12 16.5m0 0L7.5 12m4.5 4.5V3" />
            </svg>
            Download
          </button>
          <button
            onclick={() => (xmlModalOpen = false)}
            class="p-1 rounded-lg text-slate-500 hover:text-slate-200 transition-colors"
            aria-label="Close"
          >
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>

      <!-- XML content -->
      <pre class="overflow-auto flex-1 p-5 text-xs leading-relaxed text-slate-300 font-mono rounded-b-2xl"><code>{currentXml}</code></pre>
    </div>
  </div>
{/if}

<!-- Main page -->
<div class="min-h-screen bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900 text-slate-100">
  <!-- Header -->
  <header class="border-b border-slate-700/50 bg-slate-900/60 backdrop-blur-sm sticky top-0 z-10">
    <div class="max-w-4xl mx-auto px-6 py-4 flex items-center gap-3">
      <button
        onclick={onback}
        class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-slate-600 hover:border-slate-400
               text-slate-400 hover:text-slate-200 text-sm transition-colors duration-150"
      >
        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M15.75 19.5L8.25 12l7.5-7.5" />
        </svg>
        Back
      </button>
      <div class="w-8 h-8 rounded-lg bg-indigo-500 flex items-center justify-center text-white font-bold text-sm">J</div>
      <span class="font-semibold text-lg tracking-tight">Edit Document</span>
      <div class="ml-auto flex gap-2">
        <button
          onclick={() => (xmlModalOpen = true)}
          class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-sm font-medium border border-slate-600
                 hover:border-slate-400 text-slate-300 hover:text-white transition-colors duration-150"
        >
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M17.25 6.75L22.5 12l-5.25 5.25m-10.5 0L1.5 12l5.25-5.25m7.5-3l-4.5 16.5" />
          </svg>
          View XML
        </button>
        <button
          onclick={downloadXml}
          class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-sm font-medium bg-indigo-600
                 hover:bg-indigo-500 text-white transition-colors duration-150"
        >
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5M16.5 12L12 16.5m0 0L7.5 12m4.5 4.5V3" />
          </svg>
          Download .jats.xml
        </button>
      </div>
    </div>
  </header>

  <main class="max-w-4xl mx-auto px-6 py-10 space-y-6">

    <!-- Warnings -->
    {#if currentWarnings.length > 0}
      <details class="rounded-xl bg-amber-500/10 border border-amber-500/30 px-4 py-3 group">
        <summary class="cursor-pointer flex items-center gap-2 text-amber-300 font-semibold text-sm select-none list-none">
          <svg class="w-4 h-4 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126zM12 15.75h.007v.008H12v-.008z" />
          </svg>
          {currentWarnings.length} warning{currentWarnings.length > 1 ? 's' : ''}
          <svg class="w-3 h-3 ml-auto transition-transform group-open:rotate-180" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
            <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 8.25l-7.5 7.5-7.5-7.5" />
          </svg>
        </summary>
        <ul class="mt-3 space-y-1 text-amber-200/80 text-sm">
          {#each currentWarnings as w}
            <li class="flex gap-2"><span class="text-amber-500 mt-0.5">·</span>{w}</li>
          {/each}
        </ul>
      </details>
    {/if}

    <!-- Title -->
    <div class="rounded-xl bg-slate-800/60 border border-slate-700/50 p-4 space-y-2">
      <label class="text-xs font-semibold text-slate-400 uppercase tracking-wider" for="edit-title">Title</label>
      <input
        id="edit-title"
        type="text"
        bind:value={editableDoc.title}
        placeholder="Article title"
        class="w-full bg-slate-900/60 border border-slate-700 rounded-lg px-3 py-2 text-sm text-slate-100
               placeholder-slate-500 focus:outline-none focus:border-indigo-500 transition-colors"
      />
    </div>

    <!-- Authors -->
    <div class="rounded-xl bg-slate-800/60 border border-slate-700/50 p-4 space-y-3">
      <div class="flex items-center justify-between">
        <span class="text-xs font-semibold text-slate-400 uppercase tracking-wider">Authors</span>
        <button
          onclick={addAuthor}
          class="text-xs text-indigo-400 hover:text-indigo-300 transition-colors flex items-center gap-1"
        >
          <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
            <path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
          </svg>
          Add author
        </button>
      </div>
      {#if editableDoc.authors.length === 0}
        <p class="text-xs text-slate-500 italic">No authors detected.</p>
      {/if}
      {#each editableDoc.authors as author, i}
        <div class="flex gap-2">
          <input
            type="text"
            bind:value={author.name}
            placeholder="Author name"
            class="flex-1 bg-slate-900/60 border border-slate-700 rounded-lg px-3 py-2 text-sm text-slate-100
                   placeholder-slate-500 focus:outline-none focus:border-indigo-500 transition-colors"
          />
          <button
            onclick={() => removeAuthor(i)}
            class="px-2 py-2 rounded-lg border border-slate-700 hover:border-red-500/50 text-slate-500
                   hover:text-red-400 transition-colors"
            aria-label="Remove author"
          >
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      {/each}
    </div>

    <!-- Abstract -->
    <div class="rounded-xl bg-slate-800/60 border border-slate-700/50 p-4 space-y-2">
      <label class="text-xs font-semibold text-slate-400 uppercase tracking-wider" for="edit-abstract">Abstract</label>
      <textarea
        id="edit-abstract"
        bind:value={editableDoc.abstract_text}
        placeholder="Article abstract…"
        rows="5"
        class="w-full bg-slate-900/60 border border-slate-700 rounded-lg px-3 py-2 text-sm text-slate-100
               placeholder-slate-500 focus:outline-none focus:border-indigo-500 transition-colors resize-y"
      ></textarea>
    </div>

    <!-- Sections -->
    <div class="rounded-xl bg-slate-800/60 border border-slate-700/50 p-4 space-y-3">
      <div class="flex items-center justify-between">
        <span class="text-xs font-semibold text-slate-400 uppercase tracking-wider">
          Sections ({editableDoc.sections.length})
        </span>
        <button
          onclick={addSection}
          class="text-xs text-indigo-400 hover:text-indigo-300 transition-colors flex items-center gap-1"
        >
          <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
            <path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
          </svg>
          Add section
        </button>
      </div>
      {#each editableDoc.sections as section, si}
        <details class="border border-slate-700/60 rounded-lg group" open={si === 0}>
          <summary class="cursor-pointer px-3 py-2.5 flex items-center gap-2 text-sm font-medium text-slate-300 select-none list-none">
            <svg class="w-3 h-3 shrink-0 transition-transform group-open:rotate-90" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
              <path stroke-linecap="round" stroke-linejoin="round" d="M8.25 4.5l7.5 7.5-7.5 7.5" />
            </svg>
            <span class="truncate">{section.heading || `Section ${si + 1}`}</span>
            <span class="ml-auto text-xs text-slate-500">H{section.level}</span>
            <button
              onclick={(e) => { e.stopPropagation(); removeSection(si); }}
              class="p-1 rounded hover:text-red-400 text-slate-500 transition-colors"
              aria-label="Remove section"
            >
              <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </summary>
          <div class="px-3 pb-3 pt-1 space-y-2">
            <input
              type="text"
              bind:value={section.heading}
              placeholder="Section heading"
              class="w-full bg-slate-900/60 border border-slate-700 rounded-lg px-3 py-2 text-sm text-slate-100
                     placeholder-slate-500 focus:outline-none focus:border-indigo-500 transition-colors"
            />
            {#each section.body as _para, pi}
              <div class="flex gap-2">
                <textarea
                  bind:value={section.body[pi]}
                  placeholder="Paragraph text…"
                  rows="3"
                  class="flex-1 bg-slate-900/60 border border-slate-700 rounded-lg px-3 py-2 text-sm text-slate-100
                         placeholder-slate-500 focus:outline-none focus:border-indigo-500 transition-colors resize-y"
                ></textarea>
                {#if section.body.length > 1}
                  <button
                    onclick={() => removeParagraph(si, pi)}
                    class="px-2 py-2 self-start rounded-lg border border-slate-700 hover:border-red-500/50
                           text-slate-500 hover:text-red-400 transition-colors"
                    aria-label="Remove paragraph"
                  >
                    <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                      <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                  </button>
                {/if}
              </div>
            {/each}
            <button
              onclick={() => addParagraph(si)}
              class="text-xs text-slate-500 hover:text-slate-300 transition-colors flex items-center gap-1"
            >
              <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
                <path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
              </svg>
              Add paragraph
            </button>
          </div>
        </details>
      {/each}
    </div>

    <!-- References -->
    <div class="rounded-xl bg-slate-800/60 border border-slate-700/50 p-4 space-y-3">
      <span class="text-xs font-semibold text-slate-400 uppercase tracking-wider block">
        References ({editableDoc.references.length})
      </span>
      {#if editableDoc.references.length === 0}
        <p class="text-xs text-slate-500 italic">No references detected.</p>
      {/if}
      {#each editableDoc.references as ref, i}
        <div class="space-y-1">
          <div class="flex items-center justify-between">
            <span class="text-xs text-slate-500 font-mono">{ref.id}</span>
            <button
              onclick={() => removeReference(i)}
              class="text-xs text-slate-600 hover:text-red-400 transition-colors"
              aria-label="Remove reference"
            >
              <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
          <textarea
            bind:value={ref.raw_text}
            rows="2"
            class="w-full bg-slate-900/60 border border-slate-700 rounded-lg px-3 py-2 text-sm text-slate-100
                   focus:outline-none focus:border-indigo-500 transition-colors resize-y"
          ></textarea>
        </div>
      {/each}
    </div>

    <!-- Apply button -->
    <button
      onclick={applyAndPreview}
      disabled={regenerating}
      class="w-full flex items-center justify-center gap-2 px-4 py-3 rounded-xl bg-indigo-600
             hover:bg-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed font-semibold text-sm
             transition-colors duration-150"
    >
      {#if regenerating}
        <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
        </svg>
        Regenerating…
      {:else}
        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0l3.181 3.183a8.25 8.25 0 0013.803-3.7M4.031 9.865a8.25 8.25 0 0113.803-3.7l3.181 3.182m0-4.991v4.99" />
        </svg>
        Apply & Preview XML
      {/if}
    </button>

    {#if regenError}
      <div class="flex items-start gap-3 rounded-xl bg-red-500/10 border border-red-500/30 px-4 py-3.5 text-red-300 text-sm">
        <svg class="w-5 h-5 mt-0.5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126zM12 15.75h.007v.008H12v-.008z" />
        </svg>
        {regenError}
      </div>
    {/if}
  </main>
</div>
